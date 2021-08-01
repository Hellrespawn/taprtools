use super::argparse::Args;
use super::helpers;
use super::history::{Action, ActionGroup, History};
use super::validate::validate;
use crate::error::InterpreterError;
use crate::file::audio_file::{self, AudioFile};
use crate::tfmt::ast::Program;
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use crate::tfmt::semantic::SemanticAnalyzer;
use anyhow::{bail, Result};
use indicatif::{
    ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressStyle,
};
use log::warn;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(not(feature = "rayon"))]
use indicatif::ProgressIterator;

pub type PathPair = (PathBuf, PathBuf);
pub type PathPairs = Vec<PathPair>;

pub struct Rename<'a> {
    pub args: &'a Args,
}

impl<'a> Rename<'a> {
    pub fn rename<P: AsRef<Path>>(
        &mut self,
        script_name: &str,
        arguments: &[&str],
        input_folder: &P,
        output_folder: &Option<P>,
        recursive: bool,
    ) -> Result<()> {
        // FIXME Check that there are actually files to move
        // TODO? Explicitly concat cwd and relative path?
        let path = helpers::get_script(script_name, &self.args.config_folder)?;

        let program = Parser::try_from(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let audio_files = Self::get_audio_files(
            &input_folder,
            if recursive { 4 } else { 1 },
        )?;

        if audio_files.is_empty() {
            let s = format!(
                "Couldn't find any files at {}.",
                input_folder.as_ref().to_string_lossy()
            );
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        let paths =
            self.interpret_audio_files(&audio_files, &program, arguments)?;

        let (to_move, _no_move) = validate(&paths)?;

        if to_move.is_empty() {
            let s = "All files are already in the requested position.";
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        Self::preview_audio_files(&to_move, 8);

        self.rename_audio_files(&to_move, input_folder, output_folder)
    }

    pub fn get_audio_files<P: AsRef<Path>>(
        dir: &P,
        depth: u64,
    ) -> Result<Vec<Box<dyn AudioFile>>> {
        let bar = ProgressBar::new(0);

        bar.set_style(
            ProgressStyle::default_spinner()
                .template("[{pos}/{len} audio files/total files] {spinner}")
                .on_finish(ProgressFinish::AtCurrentPos),
        );
        //bar.set_length(0);
        bar.set_draw_target(ProgressDrawTarget::stdout());

        let mut audio_files = Vec::new();

        audio_file::get_audio_files(
            &mut audio_files,
            dir.as_ref(),
            depth,
            Some(&bar),
        )?;

        Ok(audio_files)
    }

    fn interpret_audio_files(
        &self,
        audio_files: &[Box<dyn AudioFile>],
        program: &Program,
        arguments: &[&str],
    ) -> Result<PathPairs> {
        let symbol_table = SemanticAnalyzer::analyze(program, arguments)?;

        let bar = ProgressBar::new(audio_files.len().try_into()?);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{pos}/{len}] {msg:<21} {wide_bar}")
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Interpreted."),
                )),
        );
        bar.set_draw_target(ProgressDrawTarget::stdout());
        bar.set_message("Interpreting files...");

        #[cfg(feature = "rayon")]
        let iter = audio_files.par_iter().progress_with(bar);

        #[cfg(not(feature = "rayon"))]
        let iter = audio_files.iter().progress_with(bar);

        let paths: std::result::Result<PathPairs, InterpreterError> = iter
            .map(|af| {
                helpers::sleep();
                let result =
                    Interpreter::new(program, &symbol_table, af.as_ref())
                        .interpret();
                //.map(|s| (PathBuf::from(af.path()), PathBuf::from(s)));

                // TODO? Why do we need to manually destructure here?
                match result {
                    Ok(s) => Ok((PathBuf::from(af.path()), PathBuf::from(s))),
                    Err(e) => Err(e),
                }
            })
            .collect();

        let paths = paths?;

        Ok(paths)
    }

    fn preview_audio_files<P: AsRef<Path>>(paths: &[(P, P)], amount: usize) {
        println!(
            "\nPreviewing {}/{} files:",
            std::cmp::min(amount, paths.len()),
            paths.len()
        );

        for (i, (_, d)) in paths.iter().enumerate() {
            if i >= amount {
                break;
            }
            println!("{}", d.as_ref().to_string_lossy())
        }

        println!();
    }

    fn create_dir_recursive<P: AsRef<Path>>(
        &self,
        path: &P,
    ) -> Result<ActionGroup> {
        let path = path.as_ref();

        if path.is_dir() | (path == Path::new("")) {
            Ok(Vec::new())
        } else if path.exists() {
            bail!(
                "Path {} exists, but isn't a directory!",
                path.to_string_lossy()
            )
        } else {
            let mut action_group = ActionGroup::new();

            if let Some(parent) = path.parent() {
                action_group.extend(self.create_dir_recursive(&parent)?)
            }

            let action = Action::CreateDir {
                path: PathBuf::from(path),
            };
            action.apply(self.args.dry_run)?;
            action_group.push(action);

            Ok(action_group)
        }
    }

    fn remove_dir_recursive<P: AsRef<Path>>(
        &self,
        root_path: &P,
        depth: u64
    ) -> Result<ActionGroup> {
        if depth == 0 {
            return Ok(Vec::new())
        }

        let mut action_group = ActionGroup::new();

        for result in std::fs::read_dir(root_path.as_ref())? {
            let path = result?.path();

            if path.is_dir() {
                action_group.extend(self.remove_dir_recursive(&path, depth - 1)?);

                let action = Action::RemoveDir{path};
                if let Ok(()) = action.apply(self.args.dry_run) {
                    action_group.push(action);
                }
            }


        }

        Ok(action_group)

    }

    fn rename_audio_files<P: AsRef<Path>>(
        &self,
        paths: &[(PathBuf, PathBuf)],
        input_folder: &P,
        output_folder: &Option<P>,
    ) -> Result<()> {
        // Absolute paths clobber existing paths when joined/pushed.
        let prefix = if let Some(folder) = output_folder {
            let folder = folder.as_ref();

            if paths.iter().any(|(_, p)| p.is_absolute()) {
                let s = format!(
                    "Absolute path found, ignoring --output-folder {}",
                    folder.to_string_lossy()
                );
                println!("{}", s);
                warn!("{}", s);
            }

            PathBuf::from(folder)
        } else {
            PathBuf::new()
        };

        let bar = ProgressBar::new(paths.len().try_into()?);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{pos}/{len}] {msg:<21} {wide_bar}")
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Renamed."),
                )),
        );
        bar.set_draw_target(ProgressDrawTarget::stdout());
        bar.set_message("Renaming files...");

        let mut action_group = ActionGroup::new();

        for (origin, destination) in paths {
            let destination = prefix.join(destination);
            // These paths are all files, so should always have at
            // least one parent.
            debug_assert!(destination.parent().is_some());

            action_group.extend(
                self.create_dir_recursive(&destination.parent().unwrap())?,
            );

            let action = Action::Rename {
                origin: PathBuf::from(origin),
                destination,
            };

            action.apply(self.args.dry_run)?;
            action_group.push(action);

            bar.inc(1);
        }

        bar.finish();

        print!("Removing empty directories... ");

        action_group.extend(self.remove_dir_recursive(input_folder, 4)?);

        println!("Done.");

        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.insert(action_group)?;

        history.save_to_path(&self.args.config_folder)?;

        Ok(())
    }
}
