use super::argparse::{Args, Subcommand};
use super::history::{History, Rename};
use super::inspector::{Inspector, Mode};
use super::strings::Strings;
use super::{argparse, helpers, logging};
use crate::error::InterpreterError;
use crate::file::audio_file::{AudioFile, MP3, OGG};
use crate::tfmt::ast::Program;
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use crate::tfmt::semantic::SemanticAnalyzer;
use anyhow::Result;
use indicatif::{
    ProgressBar, ProgressDrawTarget, ProgressFinish, ProgressIterator,
    ProgressStyle,
};
use log::info;
use std::convert::{TryFrom, TryInto};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[cfg(feature = "rayon")]
use indicatif::ParallelProgressIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// Main tfmttools entrypoint.
pub fn main<S: AsRef<OsStr>>(args: Option<&[S]>) -> Result<()> {
    match args {
        Some(args) => _main(args),
        None => _main(&std::env::args().collect::<Vec<String>>()),
    }
}

fn _main<S: AsRef<OsStr>>(args: &[S]) -> Result<()> {
    let args = argparse::parse_args(args)?;

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;

    #[cfg(feature = "rayon")]
    info!("rayon is enabled, running in parallel.");

    #[cfg(not(feature = "rayon"))]
    info!("rayon is not enabled.");

    info!("Parsed arguments:\n{:#?}", &args);

    // TODO Pretty-print errors
    TFMTTools { args: &args }.main()?;

    Ok(())
}

struct TFMTTools<'a> {
    args: &'a Args,
}

impl<'a> TFMTTools<'a> {
    fn main(&mut self) -> Result<()> {
        self.handle_command(&self.args.subcommand)
    }

    fn handle_command(&mut self, subcommand: &Subcommand) -> Result<()> {
        match subcommand {
            Subcommand::ClearHistory => self.clear_history(),
            Subcommand::ListScripts => self.list_scripts(),
            Subcommand::Redo(amount) => self.redo(*amount),
            Subcommand::Undo(amount) => self.undo(*amount),
            Subcommand::Inspect {
                script_name,
                visualize,
            } => self.inspect(script_name, *visualize),
            Subcommand::Rename {
                script_name,
                arguments,
                input_folder,
                output_folder,
                recursive,
            } => self.rename(
                script_name,
                &arguments
                    .iter()
                    .map(std::ops::Deref::deref)
                    .collect::<Vec<&str>>(),
                input_folder,
                output_folder,
                *recursive,
            ),
        }
    }

    fn clear_history(&self) -> Result<()> {
        match History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        ) {
            Ok(mut history) => history.delete()?,
            Err(err) if err.to_string().contains("Unable to load") => {
                Strings::ClearHistoryCantFindHistory.wprint()
            }

            Err(err) => Strings::ClearHistoryError(&err.to_string()).wprint(),
        }
        Ok(())
    }

    fn list_scripts(&self) -> Result<()> {
        let paths = &helpers::get_all_scripts(&self.args.config_folder);

        if paths.is_empty() {
            Strings::ListScripts.iprint()
        } else {
            for path in paths {
                Inspector::inspect(path, Mode::Short)?
            }
        }

        Ok(())
    }

    fn redo(&mut self, amount: u64) -> Result<()> {
        // Creating a new history will make history.history_action() return
        // without doing anything, thus never setting history.changed.
        // We run history.save() purely for the side effects.
        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.redo(amount)?;

        history.save_to_path(&self.args.config_folder)?;

        Ok(())
    }

    fn undo(&mut self, amount: u64) -> Result<()> {
        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.undo(amount)?;

        history.save_to_path(&self.args.config_folder)?;

        Ok(())
    }

    fn inspect(&self, name: &str, render_ast: bool) -> Result<()> {
        Inspector::inspect(
            &helpers::get_script(name, &self.args.config_folder)?,
            if render_ast { Mode::Dot } else { Mode::Long },
        )
    }

    fn rename<P: AsRef<Path>>(
        &mut self,
        script_name: &str,
        arguments: &[&str],
        input_folder: &P,
        output_folder: &Option<P>,
        recursive: bool,
    ) -> Result<()> {
        let path = helpers::get_script(script_name, &self.args.config_folder)?;

        let program = Parser::try_from(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let audio_files = TFMTTools::get_audio_files(
            &input_folder,
            if recursive { 4 } else { 1 },
        )?;

        let paths = TFMTTools::interpret_audio_files(
            &audio_files,
            &program,
            arguments,
        )?;

        debug_assert_eq!(audio_files.len(), paths.len());

        self.rename_audiofiles(&audio_files, &paths, output_folder)
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

        get_audio_files(&mut audio_files, dir.as_ref(), depth, Some(&bar))?;

        Ok(audio_files)
    }

    fn interpret_audio_files(
        audio_files: &[Box<dyn AudioFile>],
        program: &Program,
        arguments: &[&str],
    ) -> Result<Vec<PathBuf>> {
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

        let paths: std::result::Result<Vec<PathBuf>, InterpreterError> = iter
            .map(|s| {
                sleep();
                Interpreter::new(program, &symbol_table, s.as_ref())
                    .interpret()
                    .map(PathBuf::from)
            })
            .collect();

        Ok(paths?)
    }

    fn rename_audiofiles<P: AsRef<Path>>(
        &self,
        audio_files: &[Box<dyn AudioFile>],
        paths: &[PathBuf],
        output_folder: &Option<P>,
    ) -> Result<()> {
        // Absolute paths clobber existing paths when joined/pushed.
        let prefix = if let Some(folder) = output_folder {
            let folder = folder.as_ref();
            if paths.iter().any(|p| p.is_absolute()) {
                Strings::RenameIgnoringOutputFolder(folder).wprint();
            }

            PathBuf::from(folder)
        } else {
            PathBuf::new()
        };

        let bar = ProgressBar::new(audio_files.len().try_into()?);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{pos}/{len}] {msg:<21} {wide_bar}")
                .on_finish(ProgressFinish::WithMessage(
                    std::borrow::Cow::Borrowed("Renamed."),
                )),
        );
        bar.set_draw_target(ProgressDrawTarget::stdout());
        bar.set_message("Renaming files...");

        let action: Vec<Rename> = paths
            .iter()
            .zip(audio_files)
            .map(|(p, s)| Rename::new(&prefix.join(p), s.path()))
            .collect();

        action
            .iter()
            .progress_with(bar)
            .try_for_each(|r| r.apply(self.args.dry_run))?;

        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.insert(action)?;

        history.save_to_path(&self.args.config_folder)?;

        Ok(())
    }
}

#[cfg(feature = "slow-progress-bars")]
fn sleep() {
    std::thread::sleep(std::time::Duration::from_millis(200));
}

#[cfg(not(feature = "slow-progress-bars"))]
fn sleep() {}

pub fn get_audio_files(
    audio_files: &mut Vec<Box<dyn AudioFile>>,
    dir: &Path,
    depth: u64,
    bar: Option<&ProgressBar>,
) -> Result<()> {
    if depth == 0 {
        return Ok(());
    }

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(extension) = path.extension() {
                        if let Some(bar) = bar {
                            bar.inc_length(1)
                        };

                        if extension == "mp3" {
                            audio_files.push(Box::new(MP3::try_from(&path)?));
                        } else if extension == "ogg" {
                            audio_files.push(Box::new(OGG::try_from(&path)?));
                        } else {
                            continue;
                        }

                        if let Some(bar) = bar {
                            bar.inc(1)
                        };

                        sleep();
                    }
                } else if file_type.is_dir() {
                    get_audio_files(audio_files, &path, depth - 1, bar)?
                }
            }
        }
    }

    Ok(())
}
