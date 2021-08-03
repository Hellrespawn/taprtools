use super::argparse::Args;
use super::history::{Action, ActionGroup, History};
use super::validate::validate;
use crate::error::InterpreterError;
use crate::file::audio_file::{self, AudioFile};
use crate::helpers::{self, pp};
use crate::tfmt::ast::Program;
use crate::tfmt::{Interpreter, Lexer, Parser, SemanticAnalyzer};
use anyhow::{bail, Result};
use indicatif::{
    ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressStyle,
};
use log::{debug, warn};
use std::convert::TryInto;
use std::path::{Path, PathBuf};

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
        output_folder: &Option<P>,
        recursive: bool,
    ) -> Result<()> {
        // TODO? Explicitly concat cwd and relative path?
        let path = helpers::get_script(script_name, &self.args.config_folder)?;

        let program = Parser::<Lexer>::from_path(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let audio_files =
            self.get_audio_files(&input_folder, if recursive { 4 } else { 1 })?;

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
            let s = "All files are already in the requested location.";
            println!("{}", s);
            warn!("{}", s);
            return Ok(());
        }

        Self::preview_audio_files(&to_move, 8);

        self.rename_audio_files(&to_move, output_folder)
    }

    pub fn get_audio_files<P: AsRef<Path>>(
        &self,
        dir: &P,
        depth: u64,
    ) -> Result<Vec<Box<dyn AudioFile>>> {
        let bar = ProgressBar::new(0);

        let mut template =
            "[{pos}/{len} audio files/total files] {spinner}".to_string();
        template.insert_str(0, pp(self.args.preview));

        bar.set_style(
            ProgressStyle::default_spinner()
                .template(&template)
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
    ) -> Result<Vec<SrcTgtPair>> {
        let symbol_table = SemanticAnalyzer::analyze(program, arguments)?;

        let bar = ProgressBar::new(audio_files.len().try_into()?);

        let mut template = "[{pos}/{len}] {msg:<21} {wide_bar}".to_string();
        template.insert_str(0, pp(self.args.preview));

        bar.set_style(
            ProgressStyle::default_bar().template(&template).on_finish(
                ProgressFinish::WithMessage(std::borrow::Cow::Borrowed(
                    "Interpreted.",
                )),
            ),
        );
        bar.set_draw_target(ProgressDrawTarget::stdout());
        bar.set_message("Interpreting files...");

        #[cfg(feature = "rayon")]
        let path_iter = audio_files.par_iter().progress_with(bar);

        #[cfg(not(feature = "rayon"))]
        let paths_iter = audio_files.iter().progress_with(bar);

        let paths: std::result::Result<Vec<SrcTgtPair>, InterpreterError> =
            path_iter
                .map(|af| {
                    helpers::sleep();
                    let result =
                        Interpreter::new(program, &symbol_table, af.as_ref())
                            .interpret();

                    match result {
                        Ok(s) => {
                            Ok((PathBuf::from(af.path()), PathBuf::from(s)))
                        }
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
            Ok(ActionGroup::new())
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

        debug!("Common path of input: {}", common_path.to_string_lossy());
        common_path
    }

    fn rename_audio_files<P: AsRef<Path>>(
        &self,
        paths: &[(PathBuf, PathBuf)],
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

        let mut template = "[{pos}/{len}] {msg:<21} {wide_bar}".to_string();
        template.insert_str(0, pp(self.args.preview));

        bar.set_style(
            ProgressStyle::default_bar().template(&template).on_finish(
                ProgressFinish::WithMessage(std::borrow::Cow::Borrowed(
                    "Renamed.",
                )),
            ),
        );
        bar.set_draw_target(ProgressDrawTarget::stdout());
        bar.set_message("Renaming files...");

        let mut action_group = ActionGroup::new();

        for (source, target) in paths {
            let target = prefix.join(target);
            // These paths are all files, so should always have at
            // least one parent.
            debug_assert!(target.parent().is_some());

            action_group
                .extend(self.create_dir_recursive(&target.parent().unwrap())?);

            let action = Action::Rename {
                source: PathBuf::from(source),
                target,
            };

            action.apply(self.args.preview)?;
            action_group.push(action);

            bar.inc(1);
        }

        bar.finish();

        print!("{}Removing empty directories... ", pp(self.args.preview));

        action_group.extend(
            self.remove_dir_recursive(&Rename::get_common_path(paths), 4)?,
        );

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
