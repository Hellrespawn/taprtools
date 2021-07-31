use super::argparse::{Args, Subcommand};
use super::history::{History, Rename};
use super::inspector::{Inspector, Mode};
use super::{argparse, helpers, logging};
use crate::file::audiofile::get_audiofiles;
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use anyhow::Result;
use log::info;
use std::convert::{TryFrom, TryInto};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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
                println!("Can't find history file to clear!")
            }

            Err(err) => {
                println!("Error while trying to clear history!\n{}", err)
            }
        }
        Ok(())
    }

    fn list_scripts(&self) -> Result<()> {
        let paths = &helpers::get_all_scripts(&self.args.config_folder);

        if paths.is_empty() {
            println!("Couldn't find any scripts.")
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
        let depth = if recursive { 4 } else { 1 };
        let songs = get_audiofiles(&input_folder, depth)?;

        let mut intp = Interpreter::new(&program, arguments, &songs)?;

        #[cfg(feature = "rayon")]
        let mut paths: Vec<PathBuf> =
            intp.interpret()?.par_iter().map(PathBuf::from).collect();

        #[cfg(not(feature = "rayon"))]
        let mut paths: Vec<PathBuf> =
            intp.interpret()?.iter().map(PathBuf::from).collect();

        if let Some(prefix) = output_folder {
            let prefix = prefix.as_ref();

            #[cfg(feature = "rayon")]
            let iter = paths.par_iter();

            #[cfg(not(feature = "rayon"))]
            let mut iter = paths.iter();

            if iter.any(|p| p.is_absolute()) {
                println!(
                    "Absolute path found, ignoring --output-folder {}",
                    prefix.to_string_lossy()
                );
            } else {
                paths = paths
                    .into_iter()
                    .map(|p| prefix.join(p))
                    .collect::<Vec<PathBuf>>();
            }
        }

        println!("Paths:\n{:#?}", paths);

        let action: Vec<Rename> = paths
            .iter()
            .zip(&songs)
            .map(|(p, s)| Rename::new(p, s.path()))
            .collect();

        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.apply(action)?;

        history.save_to_path(&self.args.config_folder)?;

        Ok(())
    }
}
