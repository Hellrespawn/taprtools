use super::argparse::{Args, Subcommand};
use super::rename::Rename;
use super::{argparse, logging};
use crate::tfmt::visitors::{Inspector, InspectorMode};
use crate::{helpers, HISTORY_FILENAME};
use anyhow::Result;
use file_history::History;
use log::{info, warn};
use std::convert::TryInto;
use std::ffi::OsStr;

/// Main tfmttools entrypoint.
pub fn main<S: AsRef<OsStr>>(args: &[S], preview: bool) -> Result<()> {
    let args = {
        let mut args = argparse::parse_args(args)?;
        args.preview |= preview;
        args
    };

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;

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
            } => Rename {
                script_name,
                arguments: &arguments
                    .iter()
                    .map(std::ops::Deref::deref)
                    .collect::<Vec<&str>>(),
                input_folder,
                output_folder,
                config_folder: &self.args.config_folder,
                recursive: *recursive,
                preview: self.args.preview,
            }
            .rename(),
        }
    }

    fn clear_history(&self) -> Result<()> {
        match History::load_file(
            &self.args.config_folder.join(HISTORY_FILENAME),
            false,
        ) {
            Ok(mut history) => {
                if self.args.preview {
                    print!("[P] Deleted history file.");
                } else {
                    history.delete()?;
                }
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
        let paths = &helpers::get_script_paths(&self.args.config_folder);

        if paths.is_empty() {
            let s = "Couldn't find any scripts.";
            println!("{}", s);
            info!("{}", s);
        } else {
            println!("Found {} scripts:", paths.len());
            let mode = if self.args.verbosity == 0 {
                InspectorMode::Short
            } else {
                InspectorMode::Long
            };

            for path in paths {
                Inspector::inspect(path, mode)?;
            }
        }

        Ok(())
    }

    fn redo(&mut self, amount: u64) -> Result<()> {
        // Creating a new history will make history.history_action() return
        // without doing anything, thus never setting history.changed.
        // We run history.save() purely for the side effects.
        let mut history = History::load_file(
            &self.args.config_folder.join(HISTORY_FILENAME),
            false,
        )
        .unwrap_or_else(|_| History::new(false));

        if self.args.preview {
            println!("[P] Redoing {} times.", amount);
        } else {
            history.redo(amount)?;

            history.save().or_else(|_| {
                history.save_to_file(
                    &self.args.config_folder.join(HISTORY_FILENAME),
                )
            })?;
        }

        Ok(())
    }

    fn undo(&mut self, amount: u64) -> Result<()> {
        // Creating a new history will make history.history_action() return
        // without doing anything, thus never setting history.changed.
        // We run history.save() purely for the side effects.
        let mut history = History::load_file(
            &self.args.config_folder.join(HISTORY_FILENAME),
            false,
        )
        .unwrap_or_else(|_| History::new(false));

        if self.args.preview {
            println!("[P] Undoing {} times.", amount);
        } else {
            history.undo(amount)?;

            history.save().or_else(|_| {
                history.save_to_file(
                    &self.args.config_folder.join(HISTORY_FILENAME),
                )
            })?;
        }

        Ok(())
    }

    fn inspect(&self, name: &str, render_ast: bool) -> Result<()> {
        Inspector::inspect(
            &helpers::get_script_path(name, &self.args.config_folder)?,
            if render_ast {
                InspectorMode::Dot
            } else {
                InspectorMode::Long
            },
        )
    }
}
