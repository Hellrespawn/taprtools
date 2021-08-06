use super::argparse::Args;
use super::history::{Action, ActionGroup, History, MoveMode};
use super::validate::validate;
use crate::error::InterpreterError;
use crate::file::audio_file::{self, AudioFile};
use crate::helpers::{self, pp};
use crate::tfmt::ast::Program;
use crate::tfmt::{Interpreter, Lexer, Parser, SemanticAnalyzer};
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
//pub type SrcTgtPairs = Vec<SrcTgtPair>;

pub struct Rename<'a> {
    pub args: &'a Args,
}

impl<'a> Rename<'a> {
    pub fn rename<P: AsRef<Path>>(
        &mut self,
        script_name: &str,
        arguments: &[&str],
        input_folder: &P,
        output_folder: &P,
        recursive: bool,
    ) -> Result<()> {
        let path = helpers::get_script(script_name, &self.args.config_folder)?;

        let program = Parser::<Lexer>::from_path(&path)?.parse()?;

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

        let template = format!(
            "{}[{{pos}}/{{len}} audio files/total files] {{spinner}}",
            pp(self.args.preview)
        );

        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template(&template)
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

        let template = format!(
            "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
            pp(self.args.preview)
        );

        progress_bar.set_style(
            ProgressStyle::default_bar().template(&template).on_finish(
                ProgressFinish::WithMessage(std::borrow::Cow::Borrowed(
                    "Interpreted.",
                )),
            ),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Interpreting files...");

        #[cfg(feature = "rayon")]
        let path_iter = audio_files.par_iter().progress_with(progress_bar);

        #[cfg(not(feature = "rayon"))]
        let paths_iter = audio_files.iter().progress_with(bar);

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

        let template = format!(
            "{}[{{pos}}/{{len}}] {{msg:<21}} {{wide_bar}}",
            pp(self.args.preview)
        );

        progress_bar.set_style(
            ProgressStyle::default_bar().template(&template).on_finish(
                ProgressFinish::WithMessage(std::borrow::Cow::Borrowed(
                    "Renamed.",
                )),
            ),
        );
        progress_bar.set_draw_target(ProgressDrawTarget::stdout());
        progress_bar.set_message("Renaming files...");

        let mut action_group = ActionGroup::new();

        // TODO Test this somehow?
        // These paths are all files, so should always have at least one
        // component, making unwrap() safe.
        debug_assert!(
            paths.iter().next().is_some()
                && paths.iter().next().unwrap().0.components().next().is_some()
        );
        debug_assert!(
            paths.iter().next().is_some()
                && paths.iter().next().unwrap().1.components().next().is_some()
        );

        // At this point, source is canonicalized by {MP3, OGG}::try_from
        // and target is canonicalized by rename::interpret_destinations.
        // FIXME fix this
        let move_mode = if paths.iter().any(|(src, tgt)| {
            src.components().next().unwrap() == tgt.components().next().unwrap()
        }) {
            info!("Renaming files.");
            MoveMode::Rename
        } else {
            info!("Copying and removing files.");
            MoveMode::CopyRemove
        };

        for (source, target) in paths {
            // These paths are all files, so should always have at
            // least one parent, making unwrap() safe.
            debug_assert!(target.parent().is_some());

            action_group
                .extend(self.create_dir_recursive(&target.parent().unwrap())?);

            let action = Action::Move {
                source: PathBuf::from(source),
                target: PathBuf::from(target),
                mode: move_mode,
            };

            action.apply(self.args.preview)?;
            action_group.push(action);

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
