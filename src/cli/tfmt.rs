use super::argparse::{Args, Subcommand};
use super::history::History;
use super::inspector::{Inspector, Mode};
use super::rename::Rename;
use super::{argparse, helpers, logging};
use anyhow::Result;
use log::{info, warn};
use std::convert::TryInto;
use std::ffi::OsStr;

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
    TFMTTools { args: &args }.handle_command()?;

    info!("Program execution complete. Have a nice day!");

    Ok(())
}

struct TFMTTools<'a> {
    args: &'a Args,
}

impl<'a> TFMTTools<'a> {
    pub fn handle_command(&mut self) -> Result<()> {
        match &self.args.subcommand {
            Subcommand::ClearHistory => self.clear_history(),
            Subcommand::ListScripts => self.list_scripts(),
            Subcommand::Redo(amount) => self.redo(*amount),
            Subcommand::Undo(amount) => self.undo(*amount),
            Subcommand::Inspect {
                script_name,
                render_ast,
            } => self.inspect(script_name, *render_ast),
            Subcommand::Rename {
                script_name,
                arguments,
                input_folder,
                output_folder,
                recursive,
            } => {
                let mut rename = Rename { args: self.args };

                rename.rename(
                    script_name,
                    &arguments
                        .iter()
                        .map(std::ops::Deref::deref)
                        .collect::<Vec<&str>>(),
                    input_folder,
                    output_folder,
                    *recursive,
                )
            }
        }
    }

    fn clear_history(&self) -> Result<()> {
        match History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        ) {
            Ok(mut history) => {
                history.delete()?;
            }
            Err(err) if err.to_string().contains("Unable to load") => {
                let s = "Can't find history file to clear!";
                println!("{}", s);
                warn!("{}", s);
            }

            Err(err) => {
                let s =
                    format!("Error while trying to clear history!\n{}", err);
                println!("{}", s);
                warn!("{}", s);
            }
        }
        Ok(())
    }

    fn list_scripts(&self) -> Result<()> {
        let paths = &helpers::get_scripts(&self.args.config_folder);

        if paths.is_empty() {
            let s = "Couldn't find any scripts.";
            println!("{}", s);
            info!("{}", s);
        } else {
            println!("Found {} scripts:", paths.len());
            let mode = if self.args.verbosity == 0 {
                Mode::Short
            } else {
                Mode::Long
            };

            for path in paths {
                Inspector::inspect(path, mode)?
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

        history
            .save()
            .or_else(|_| history.save_to_path(&self.args.config_folder))?;

        Ok(())
    }

    fn undo(&mut self, amount: u64) -> Result<()> {
        let mut history = History::load_from_path(
            self.args.dry_run,
            &self.args.config_folder,
        )
        .unwrap_or_default();

        history.undo(amount)?;

        history
            .save()
            .or_else(|_| history.save_to_path(&self.args.config_folder))?;

        Ok(())
    }

    fn inspect(&self, name: &str, render_ast: bool) -> Result<()> {
        Inspector::inspect(
            &helpers::get_script(name, &self.args.config_folder)?,
            if render_ast { Mode::Dot } else { Mode::Long },
        )
    }
}
