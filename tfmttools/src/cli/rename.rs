use super::argparse::Args;
use super::history::{Action, ActionGroup, History, MoveMode};
use super::validate::validate;
use crate::file::audio_file::{self, AudioFile};
use crate::helpers::{self, pp};
use crate::tfmt::ast::Program;
use crate::tfmt::error::InterpreterError;
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use crate::tfmt::semantic::SemanticAnalyzer;
use crate::{PREVIEW_AMOUNT, RECURSION_DEPTH};
use anyhow::{bail, Result};
use indicatif::{
    ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressStyle,
};
use log::{debug, info, warn};
use std::convert::TryInto;
use std::path::{Path, PathBuf};
use std::sync::Once;

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;

/// Intermediate representation during interpreting.
pub type SrcTgtPair = (PathBuf, PathBuf);

pub struct Rename<'a> {
    pub args: &'a Args,
}

impl<'a> Rename<'a> {
    pub fn rename<P, Q>(
        &mut self,
        script_name: &str,
        arguments: &[&str],
        input_folder: &P,
        output_folder: &Q,
        recursive: bool,
    ) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let path = helpers::get_script(script_name, &self.args.config_folder)?;

        let input_text =
            helpers::normalize_newlines(&std::fs::read_to_string(path)?);
        let program = Parser::from_string(&input_text)?.parse()?;

        let audio_files = self.get_audio_files(
            &input_folder,
            if recursive { RECURSION_DEPTH } else { 1 },
        )?;

        if audio_files.is_empty() {
            let s = format!(
                "Couldn't find any files at {}.",
                input_folder.as_ref().display()
            );
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        let paths = self.interpret_destinations(
            &audio_files,
            output_folder,
            &program,
            arguments,
        )?;

        let (to_move, _no_move) = validate(&paths)?;

        if to_move.is_empty() {
            let s = "All files are already in the requested location.";
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        Self::preview_audio_files(&to_move, PREVIEW_AMOUNT);

        self.rename_audio_files(&to_move)
    }

    pub fn get_audio_files<P: AsRef<Path>>(
        &self,
        dir: &P,
        depth: u64,
    ) -> Result<Vec<Box<dyn AudioFile>>> {
        let progress_bar = ProgressBar::new(0);

        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template(&format!(
                    "{}[{{pos}}/{{len}} audio files/total files] {{spinner}}",
                    pp(self.args.preview)
                ))
                .on_finish(ProgressFinish::AtCurrentPos),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());

        let audio_files = audio_file::get_audio_files(
            dir.as_ref(),
            depth,
            Some(&progress_bar),
        )?;

        Ok(audio_files)
    }

    // FIXME Interpreting bar prints double
    fn interpret_destinations<P: AsRef<Path>>(
        &self,
        audio_files: &[Box<dyn AudioFile>],
        output_folder: &P,
        program: &Program,
        arguments: &[&str],
    ) -> Result<Vec<SrcTgtPair>> {
        let output_folder = output_folder.as_ref();

        let absolute = Once::new();

        let symbol_table = SemanticAnalyzer::analyze(program, arguments)?;

        let progress_bar = ProgressBar::new(audio_files.len().try_into()?);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
                    pp(self.args.preview)
                ))
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Interpreted."),
                )),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Interpreting files...");

        #[cfg(feature = "rayon")]
        let path_iter = audio_files.par_iter().progress_with(progress_bar);

        #[cfg(not(feature = "rayon"))]
        let path_iter = audio_files.iter().progress_with(progress_bar);

        type IResult = std::result::Result<Vec<SrcTgtPair>, InterpreterError>;

        // Pushing/joining an absolute path onto another path clobbers that
        // path. Pushing/joining a relative path onto an empty path overwrites
        // it entirely. Therefore we can join output_folder unconditionally.

        let paths = path_iter
            .map(|af| {
                let result =
                    Interpreter::new(program, &symbol_table, af.as_ref())
                        .interpret();

                match result {
                    Ok(s) => {
                        let p = PathBuf::from(s);
                        if p.is_absolute() {
                            absolute.call_once(|| {})
                        }
                        Ok((PathBuf::from(af.path()), output_folder.join(p)))
                    }
                    Err(e) => Err(e),
                }
            })
            .collect::<IResult>()?;

        if absolute.is_completed() {
            let s = format!(
                "Absolute path found, ignoring --output-folder {}",
                output_folder.display()
            );
            println!("{}", s);
            warn!("{}", s);
        }

        Ok(paths)
    }

    fn preview_audio_files<P: AsRef<Path>>(paths: &[(P, P)], amount: usize) {
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
            println!("{}", d.as_ref().display())
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
                action_group.extend(self.create_dir_recursive(&parent)?)
            }

            let action = Action::CreateDir {
                path: PathBuf::from(path),
            };
            action.apply(self.args.preview)?;
            action_group.push(action);

            Ok(action_group)
        }
    }

    fn remove_dir_recursive<P: AsRef<Path>>(
        &self,
        root_path: &P,
        depth: u64,
    ) -> Result<ActionGroup> {
        if depth == 0 {
            return Ok(ActionGroup::new());
        }

        let mut action_group = ActionGroup::new();

        for result in std::fs::read_dir(root_path.as_ref())? {
            let path = result?.path();

            if path.is_dir() {
                action_group
                    .extend(self.remove_dir_recursive(&path, depth - 1)?);

                let action = Action::RemoveDir { path };
                if let Ok(()) = action.apply(self.args.preview) {
                    action_group.push(action);
                }
            }
        }

        Ok(action_group)
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
                    new_common_path.push(a)
                } else {
                    break;
                }
            }
            common_path = new_common_path;
        }

        debug!("Common path of input: {}", common_path.display());
        common_path
    }

    fn rename_audio_files(&self, paths: &[(PathBuf, PathBuf)]) -> Result<()> {
        let progress_bar = ProgressBar::new(paths.len().try_into()?);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
                    pp(self.args.preview)
                ))
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Renamed."),
                )),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Renaming files...");

        let mut action_group = ActionGroup::new();

        let mut move_mode = MoveMode::Rename;

        for (source, target) in paths {
            // These paths are all files, so should always have at
            // least one parent, making unwrap() safe.
            debug_assert!(target.parent().is_some());

            action_group
                .extend(self.create_dir_recursive(&target.parent().unwrap())?);

            action_group.push({
                let action = Action::new_move(source, target, move_mode);

                if let Err(err) = action.apply(self.args.preview) {
                    // Can't rename across filesystem boundaries. Checks for
                    // the appropriate error and changes the mode henceforth.
                    // Error codes are correct on Windows 10 20H2 and Arch
                    // Linux.

                    #[cfg(windows)]
                    let condition = err.to_string().contains("os error 17");

                    #[cfg(unix)]
                    let condition = err.to_string().contains("os error 18");

                    if condition {
                        info!("Changing mode to copy/remove");
                        move_mode = MoveMode::CopyRemove;

                        let action =
                            Action::new_move(source, target, move_mode);

                        action.apply(self.args.preview)?;
                        action
                    } else {
                        bail!(err)
                    }
                } else {
                    action
                }
            });

            progress_bar.inc(1);
        }

        progress_bar.finish();

        print!("{}Removing empty directories... ", pp(self.args.preview));

        action_group.extend(self.remove_dir_recursive(
            &Rename::get_common_path(paths),
            RECURSION_DEPTH,
        )?);

        println!("Done.");

        let mut history = History::load_from_config(
            self.args.preview,
            &self.args.config_folder,
        )
        .unwrap_or_else(|_| History::new(self.args.preview));

        history.insert(action_group)?;

        history
            .save()
            .or_else(|_| history.save_to_path(&self.args.config_folder))?;

        Ok(())
    }
}
