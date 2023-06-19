mod validate;

use crate::cli::{ui, Config};
use crate::file::AudioFile;
use crate::script::Script;
use crate::tapr::get_tapr_environment;
use anyhow::{anyhow, bail, Result};
use file_history::{Action, History, HistoryError};
use indicatif::ProgressIterator;
use std::fs;
use std::path::{Path, PathBuf};
use tapr::{Interpreter, Value};
use validate::validate_actions;

pub(crate) const FORBIDDEN_CHARACTERS: [char; 10] =
    ['<', '>', ':', '\'', '|', '?', '*', '~', '/', '\\'];

pub(crate) fn rename(
    preview: bool,
    config: &Config,
    recursion_depth: usize,
    name: &str,
    arguments: &[String],
) -> Result<()> {
    let mut history = History::load(config.path(), Config::HISTORY_NAME)?;

    let mut script = config.get_script(name)?;

    let files = gather_files(recursion_depth)?;

    script.add_arguments_to_node(arguments)?;

    let actions = interpret_files(&script, files)?;

    if actions.is_empty() {
        println!("There are no audio files to rename.");
        Ok(())
    } else {
        let (actions, _filtered_actions) = partition_actions(actions);

        validate_actions(&actions)?;

        let common_path = get_common_path(&actions);

        perform_actions(
            preview,
            recursion_depth,
            &common_path,
            &mut history,
            actions,
        )
    }
}
fn gather_files(recursion_depth: usize) -> Result<Vec<AudioFile>> {
    let path = std::env::current_dir()?;

    let spinner = ui::AudioFileSpinner::new(
        "audio files",
        "total files",
        "Gathering files...",
    );

    let paths = Config::search_path(
        &path,
        recursion_depth,
        &|p| {
            p.extension().map_or(false, |extension| {
                for supported_extension in AudioFile::SUPPORTED_EXTENSIONS {
                    if extension == supported_extension {
                        return true;
                    }
                }

                false
            })
        },
        Some(&spinner),
    );

    spinner.finish("Gathered files.");

    paths.iter().map(|p| AudioFile::new(p)).collect()
}

fn interpret_files(
    script: &Script,
    files: Vec<AudioFile>,
) -> Result<Vec<Action>> {
    let bar = ui::create_progressbar(
        files.len() as u64,
        "Interpreting files...",
        "Interpreted files.",
        false,
    );

    let actions: Result<Vec<Action>> = files
        .into_iter()
        .progress_with(bar)
        .map(|audiofile| action_from_file(script, audiofile))
        .collect();

    actions
}

fn get_common_path(actions: &[Action]) -> PathBuf {
    debug_assert!(!actions.is_empty());

    // We have already returned if no files were found, so this index
    // should be safe.
    let (common_path, _) = actions[0].get_src_tgt_unchecked();
    let mut common_path = common_path.to_path_buf();

    for action in actions {
        let (path, _) = action.get_src_tgt_unchecked();

        let mut new_common_path = PathBuf::new();

        for (left, right) in path.components().zip(common_path.components()) {
            if left == right {
                new_common_path.push(left);
            } else {
                break;
            }
        }
        common_path = new_common_path;
    }

    common_path
}

fn action_from_file(script: &Script, audiofile: AudioFile) -> Result<Action> {
    let source = audiofile.path().to_owned();

    // We already know this is a file with either an "mp3" or "ogg"
    // extension, so we unwrap safely.
    debug_assert!(source.extension().is_some());

    let extension = audiofile.extension().to_owned();

    let mut intp = create_interpreter(audiofile);
    let string = run_interpreter(script, &mut intp)?;

    let target = create_target_path_from_string(&string, &extension)?;

    let action = Action::mv(source, target);

    Ok(action)
}

fn create_interpreter(audiofile: AudioFile) -> Interpreter<'static> {
    let mut intp = Interpreter::default();

    let env = get_tapr_environment(audiofile);

    intp.push_environment(env);

    intp
}

fn run_interpreter(script: &Script, intp: &mut Interpreter) -> Result<String> {
    let value = script.accept(intp)?;

    let Value::List(segments) = value else {
        bail!("Script did not return list of segments.")
    };

    let string = segments
        .into_iter()
        .map(|v| {
            if let Value::String(segment) = v {
                Ok(replace_invalid_chars(segment))
            } else {
                Err(anyhow!(
                    "List of segments contained values other than strings."
                ))
            }
        })
        .collect::<Result<Vec<_>>>()?
        .join(std::path::MAIN_SEPARATOR_STR);

    Ok(string)
}

fn create_target_path_from_string(
    string: &str,
    extension: &str,
) -> Result<PathBuf> {
    let target_path = PathBuf::from(format!("{string}.{extension}"));

    // If target_path has an absolute path, join will clobber the current_dir,
    // so this is always safe.
    let target = std::env::current_dir()?.join(target_path);

    Ok(target)
}

fn replace_invalid_chars(string: String) -> String {
    FORBIDDEN_CHARACTERS
        .iter()
        .fold(string, |string, char| string.replace(*char, ""))
}

fn partition_actions(actions: Vec<Action>) -> (Vec<Action>, Vec<Action>) {
    actions.into_iter().partition(|action| {
        let (source, target) = action.get_src_tgt_unchecked();
        source != target
    })
}

fn perform_actions(
    preview: bool,
    recursion_depth: usize,
    common_path: &Path,
    history: &mut History,
    actions: Vec<Action>,
) -> Result<()> {
    ui::print_actions_preview(
        &actions,
        crate::cli::Args::DEFAULT_PREVIEW_AMOUNT,
    );

    move_files(preview, history, actions)?;

    clean_up_source_dirs(preview, history, common_path, recursion_depth)?;

    history.save()?;

    Ok(())
}

fn move_files(
    preview: bool,
    history: &mut History,
    actions: Vec<Action>,
) -> Result<()> {
    let bar = ui::create_progressbar(
        actions.len() as u64,
        "Moving files...",
        "Moved files.",
        preview,
    );

    for action in actions.into_iter().progress_with(bar) {
        let (_, target) = action.get_src_tgt_unchecked();
        // Actions target are all files, and always have a parent.

        debug_assert!(target.parent().is_some());
        create_dir(preview, history, target.parent().unwrap())?;
        if !preview {
            history.apply(action)?;
        }
    }

    Ok(())
}

fn create_dir(preview: bool, history: &mut History, path: &Path) -> Result<()> {
    if path.is_dir() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        create_dir(preview, history, parent)?;
    }

    let action = Action::mkdir(path);

    if !preview {
        history.apply(action)?;
    }

    Ok(())
}

fn clean_up_source_dirs(
    preview: bool,
    history: &mut History,
    common_path: &Path,
    recursion_depth: usize,
) -> Result<()> {
    let dirs = gather_dirs(common_path, recursion_depth);

    let actions: Vec<Action> = dirs.into_iter().map(Action::rmdir).collect();

    if !preview {
        for action in actions {
            remove_dir(history, action)?;
        }
    }

    let pp = if preview { Config::PREVIEW_PREFIX } else { "" };

    println!("{pp}Removed leftover folders.");

    Ok(())
}

fn gather_dirs(path: &Path, depth: usize) -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if depth == 0 {
        return dirs;
    }

    for entry in fs::read_dir(path).into_iter().flatten().flatten() {
        let dir = entry.path();
        if dir.is_dir() {
            dirs.extend(gather_dirs(&dir, depth - 1));
            dirs.push(dir);
        }
    }

    dirs
}

fn remove_dir(history: &mut History, action: Action) -> Result<()> {
    let result = history.apply(action);

    if let Err(err) = result {
        let mut is_expected_error = false;

        if let HistoryError::IO(io_error) = &err {
            if let Some(error_code) = io_error.raw_os_error() {
                #[cfg(windows)]
                // https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
                // 145: Directory not empty
                let expected_code = 145;

                // https://nuetzlich.net/errno.html
                // 39: Directory not empty
                #[cfg(unix)]
                let expected_code = 39;

                if error_code == expected_code {
                    is_expected_error = true;
                }
            }
        }

        if !is_expected_error {
            return Err(err.into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use assert_fs::TempDir;

    #[test]
    fn test_remove_dir_error_codes() -> Result<()> {
        let tempdir = TempDir::new()?;

        let test_folder = tempdir.path().join("test_folder");
        let test_file = test_folder.join("test.file");

        #[cfg(windows)]
        let expected_code = 145;

        #[cfg(unix)]
        let expected_code = 39;

        fs::create_dir(&test_folder)?;
        fs::write(test_file, "")?;
        let result = fs::remove_dir(test_folder);

        if let Err(err) = result {
            if let Some(error_code) = err.raw_os_error() {
                assert_eq!(error_code, expected_code);
                Ok(())
            } else {
                Err(err.into())
            }
        } else {
            Ok(())
        }
    }
}
