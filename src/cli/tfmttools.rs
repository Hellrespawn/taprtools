use super::argparse::{Args, Subcommand};
use super::config;
use super::inspector::{Inspector, Mode};
use super::{argparse, logging};
use crate::tfmt::parser::Parser;
use crate::tfmt::semantic::SemanticAnalyzer;
use crate::tfmt::interpreter::Interpreter;
use crate::error::InterpreterError;
use super::rename::get_audiofiles;
use anyhow::{bail, Result};
use log::info;
use std::convert::TryInto;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

/// Main tfmttools entrypoint.
pub fn main() -> Result<()> {
    let args = argparse::parse_args()?;

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;
    info!("Parsed arguments:\n{:#?}", &args);

    // TODO Pretty-print errors
    TFMTTools { args }.main()
}

struct TFMTTools {
    args: Args,
}

impl TFMTTools {
    fn main(&self) -> Result<()> {
        self.handle_command(&self.args.subcommand)
    }

    fn handle_command(&self, subcommand: &Subcommand) -> Result<()> {
        match subcommand {
            Subcommand::ListScripts => self.list_scripts(),
            Subcommand::Inspect(name) => self.inspect(name),
            Subcommand::Redo(amount) => self.redo(*amount),
            Subcommand::Undo(amount) => self.undo(*amount),
            Subcommand::Rename {
                script_name,
                arguments,
                input_folder,
                recursive,
            } => self.rename(
                script_name,
                arguments,
                input_folder,
                *recursive,
            ),
        }
    }

    fn list_scripts(&self) -> Result<()> {
        let iter = &config::get_all_scripts();
        let paths: Vec<&PathBuf> = iter.values().collect();

        if paths.is_empty() {
            println!("Couldn't find any scripts.")
        } else {
            for path in paths {
                Inspector::inspect(path, Mode::Short)?
            }
        }

        Ok(())
    }

    fn inspect(&self, name: &str) -> Result<()> {
        Inspector::inspect(
            &config::get_script(name)?,
            Mode::Dot,
        )
    }

    fn redo(&self, amount: u64) -> Result<()> {
        bail!("Redo({}) is unimplemented!", amount)
    }

    fn undo(&self, amount: u64) -> Result<()> {
        bail!("Undo({}) is unimplemented!", amount)
    }

    fn rename(
        &self,
        script_name: &str,
        arguments: &[String],
        input_folder: &Path,
        recursive: bool,
    ) -> Result<()> {
        let path = config::get_script(script_name)?;

        let program = Parser::try_from(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let depth = if recursive { 4 } else { 1 };
        let songs = get_audiofiles(input_folder, depth)?;

        let symbol_table = SemanticAnalyzer::analyze(&program, arguments)?;

        let paths: std::result::Result<Vec<String>, InterpreterError> = songs.into_iter().map(|s| Interpreter::new(s, &symbol_table).interpret(&program)).collect();

        println!("Paths:\n{:#?}", paths?);

        Ok(())
    }
}
