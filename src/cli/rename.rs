use super::validate::validate;
use crate::file::audio_file::{self, AudioFile};
use crate::helpers::{self, pp};
use crate::tfmt::error::InterpreterError;
use crate::tfmt::visitors::Interpreter;
use crate::{HISTORY_FILENAME, PREVIEW_AMOUNT, RECURSION_DEPTH};
use anyhow::{bail, Result};
use file_history::{Action, ActionGroup, History};
use indicatif::{
    ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressIterator,
    ProgressStyle,
};
use log::{debug, warn};
use std::convert::TryInto;
use std::path::{Path, PathBuf};

/// Intermediate representation during interpreting.
pub type SrcTgtPair = (PathBuf, PathBuf);

pub(crate) struct Rename<'a> {
    pub(crate) script_name: &'a str,
    pub(crate) arguments: &'a [&'a str],
    pub(crate) input_folder: &'a Path,
    pub(crate) output_folder: &'a Path,
    pub(crate) config_folder: &'a Path,
    pub(crate) recursive: bool,
    pub(crate) preview: bool,
}

impl<'a> Rename<'a> {
    pub(crate) fn rename(&self) -> Result<()> {
        // Get files
        let input_text = helpers::normalize_newlines(&std::fs::read_to_string(
            helpers::get_script_path(self.script_name, &self.config_folder)?,
        )?);

        let progress_bar = ProgressBar::new(0);

        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template(&format!(
                    "{}[{{pos}}/{{len}} audio files/total files] {{spinner}}",
                    pp(self.preview)
                ))
                .on_finish(ProgressFinish::AtCurrentPos),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());

        let audio_files = audio_file::get_audio_files(
            &self.input_folder,
            if self.recursive { RECURSION_DEPTH } else { 1 },
            Some(&progress_bar),
        )?;

        if audio_files.is_empty() {
            let s = format!(
                "Couldn't find any files at {}.",
                self.input_folder.display()
            );
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        // Interpret files
        let progress_bar = ProgressBar::new(audio_files.len().try_into()?);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
                    pp(self.preview)
                ))
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Interpreted."),
                )),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Interpreting files...");

        let mut intp = Interpreter::new(&input_text, self.arguments)?;

        let paths =
            self.interpret_destinations(&audio_files, &mut intp, progress_bar)?;

        // TODO? Validate that *all* paths are absolute?
        if paths.iter().any(|p| p.1.is_absolute()) {
            let s = format!(
                "Absolute path found, ignoring --output-folder {}",
                self.output_folder.display()
            );
            println!("{}", s);
            warn!("{}", s);
        }

        // Validate files
        let (to_move, _no_move) = validate(&paths)?;

        if to_move.is_empty() {
            let s = "All files are already in the requested location.";
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        Self::print_audio_files_preview(&to_move, PREVIEW_AMOUNT);

        // Rename files
        let progress_bar = ProgressBar::new(paths.len().try_into()?);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
                    pp(self.preview)
                ))
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Renamed."),
                )),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Renaming files...");

        let mut action_group = ActionGroup::new();

        self.rename_audio_files(&to_move, &mut action_group, &progress_bar)?;

        print!("{}Removing empty directories... ", pp(self.preview));

        self.remove_dir_recursive(
            &Rename::get_common_path(&paths),
            RECURSION_DEPTH,
            &mut action_group,
        )?;

        println!("Done.");

        let mut history = History::load_file(
            &self.config_folder.join(HISTORY_FILENAME),
            false,
        )
        .unwrap_or_else(|_| History::new(false));

        if !self.preview {
            history.insert(action_group)?;

            history.save().or_else(|_| {
                history.save_to_file(&self.config_folder.join(HISTORY_FILENAME))
            })?;
        }

        Ok(())
    }

    fn interpret_destinations(
        &self,
        audio_files: &'a [Box<dyn AudioFile>],
        intp: &'a mut Interpreter,
        progress_bar: ProgressBar,
    ) -> Result<Vec<SrcTgtPair>> {
        type IResult = std::result::Result<Vec<SrcTgtPair>, InterpreterError>;

        // Pushing/joining an absolute path onto another path clobbers that
        // path. Pushing/joining a relative path onto an empty path overwrites
        // it entirely. Therefore we can join output_folder unconditionally.

        let paths = audio_files
            .iter()
            .progress_with(progress_bar)
            .map(|af| match intp.interpret(af.as_ref()) {
                Ok(s) => {
                    Ok((PathBuf::from(af.path()), self.output_folder.join(s)))
                }
                Err(e) => Err(e),
            })
            .collect::<IResult>()?;

        Ok(paths)
    }

    fn print_audio_files_preview<P: AsRef<Path>>(
        paths: &[(P, P)],
        amount: usize,
    ) {
        let length = paths.len();

        println!(
            "\nPreviewing {} files:",
            if length > amount {
                format!("{}/{}", std::cmp::min(amount, paths.len()), length)
            } else {
                length.to_string()
            }
        );

        for (i, (_, d)) in paths.iter().enumerate() {
            if i >= amount {
                break;
            }
            println!("{}", d.as_ref().display());
        }

        println!();
    }

    fn create_dir_recursive<P: AsRef<Path>>(
        &self,
        path: &P,
    ) -> Result<ActionGroup> {
        let path = path.as_ref();

        if path.is_dir() | (path == Path::new("")) {
            Ok(ActionGroup::new())
        } else if path.exists() {
            bail!("Path {} exists, but isn't a directory!", path.display())
        } else {
            let mut action_group = ActionGroup::new();

            if let Some(parent) = path.parent() {
                action_group.extend(self.create_dir_recursive(&parent)?);
            }

            let action = Action::CreateDir(PathBuf::from(path));

            if !self.preview {
                action.apply()?;
                action_group.push(action);
            }

            Ok(action_group)
        }
    }

    fn remove_dir_recursive<P: AsRef<Path>>(
        &self,
        root_path: &P,
        depth: u64,
        action_group: &mut ActionGroup,
    ) -> Result<()> {
        // TODO? Add a spinner and counter here?
        if depth == 0 {
            return Ok(());
        }
        for result in std::fs::read_dir(root_path.as_ref())? {
            let path = result?.path();

            if path.is_dir() {
                self.remove_dir_recursive(&path, depth - 1, action_group)?;

                let action = Action::RemoveDir(path);

                if !self.preview {
                    if let Ok(()) = action.apply() {
                        action_group.push(action);
                    }
                }
            }
        }

        Ok(())
    }

    fn get_common_path(paths: &[(PathBuf, PathBuf)]) -> PathBuf {
        debug_assert!(!paths.is_empty());

        // We have already returned if no files were found, so this index
        // should be safe.
        let (mut common_path, _) = paths[0].clone();

        for (path, _) in paths {
            let mut new_common_path = PathBuf::new();

            for (a, b) in path.components().zip(common_path.components()) {
                if a == b {
                    new_common_path.push(a);
                } else {
                    break;
                }
            }
            common_path = new_common_path;
        }

        debug!("Common path of input: {}", common_path.display());
        common_path
    }

    fn rename_audio_files(
        &self,
        paths: &[(PathBuf, PathBuf)],
        action_group: &mut ActionGroup,
        progress_bar: &ProgressBar,
    ) -> Result<()> {
        // TODO Revert any actions taken if there was an error?
        for (source, target) in paths {
            // These paths are all files, so should always have at
            // least one parent, making unwrap() safe.
            debug_assert!(target.parent().is_some());

            action_group
                .extend(self.create_dir_recursive(&target.parent().unwrap())?);

            if !self.preview {
                action_group.push({
                    let action = Action::move_file(source, target);
                    action.apply()?;
                    action
                });
            }

            progress_bar.inc(1);
        }

        progress_bar.finish();

        Ok(())
    }
}
