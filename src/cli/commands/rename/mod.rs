mod validate;

use crate::cli::{ui, Config};
use crate::file::AudioFile;
use anyhow::Result;
use file_history::{Action, History, HistoryError};
use indicatif::ProgressIterator;
use std::path::{Path, PathBuf};
use tfmt::{Interpreter, Script, SymbolTable};
use validate::validate_actions;

pub(crate) fn rename(
    preview: bool,
    config: &Config,
    recursion_depth: usize,
    name: &str,
    arguments: &[String],
) -> Result<()> {
    let history_path = config.get_history_path();
    let mut history = History::load(&history_path)?;

    let script = config.get_script(name)?;

    let symbol_table = SymbolTable::new(&script, arguments)?;

    let files = gather_files(recursion_depth)?;

    let actions = interpret_files(script, symbol_table, &files)?;

    validate_actions(&actions)?;

    let common_path = get_common_path(&actions);

    let (actions, _filtered_actions) = partition_actions(actions);

    if actions.is_empty() {
        println!("There are no actions to perform.");
        Ok(())
    } else {
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
    script: Script,
    symbol_table: SymbolTable,
    files: &[AudioFile],
) -> Result<Vec<Action>> {
    let mut interpreter = Interpreter::new(script, symbol_table)?;

    let bar = ui::create_progressbar(
        files.len() as u64,
        "Interpreting files...",
        "Interpreted files.",
        false,
    );

    let actions: Result<Vec<Action>> = files
        .iter()
        .progress_with(bar)
        .map(|audiofile| action_from_file(&mut interpreter, audiofile))
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

fn action_from_file(
    interpreter: &mut Interpreter,
    audiofile: &AudioFile,
) -> Result<Action> {
    let string = interpreter.interpret(audiofile.tags())?;

    let source = audiofile.path();

    // We already know this is a file with either an "mp3" or "ogg"
    // extension, so we unwrap safely.
    debug_assert!(source.extension().is_some());
    let extension = source.extension().unwrap();

    let target = std::env::current_dir()?
        .join(string)
        .with_extension(extension);

    let action = Action::mv(source, target);

    Ok(action)
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

    let result = move_files(preview, history, actions);

    // FIXME Handle nested error somehow.
    if result.is_err() {
        history.rollback()?;
    } else {
        let nested_result = clean_up_source_dirs(
            preview,
            history,
            common_path,
            recursion_depth,
        );

        history.save()?;

        nested_result?;
    }

    result
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
        if !preview {
            let (_, target) = action.get_src_tgt_unchecked();
            // Actions target are all files, and always have a parent.

            debug_assert!(target.parent().is_some());
            create_dir(preview, history, target.parent().unwrap())?;
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

    history.apply(action)?;

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

    for entry in std::fs::read_dir(path).into_iter().flatten().flatten() {
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
                // FIXME Confirm that this error code is the same on unix
                // Directory not empty
                if error_code == 145 {
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
