use super::argparse::{Args, Subcommand};
use crate::cli::{argparse, logging};
use anyhow::Result;
use log::info;
use std::convert::TryInto;

/// Main tfmttools entrypoint.
pub fn main() -> Result<()> {
    let args = argparse::parse_args()?;

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;
    info!("Parsed arguments:\n{:#?}", &args);

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
            Subcommand::Undo { amount } => self.undo(*amount),
            Subcommand::Redo { amount } => self.redo(*amount),
            Subcommand::Inspect { name } => self.inspect(name),
            Subcommand::Rename {
                name,
                arguments,
                recursive,
            } => self.rename(name, arguments.as_ref(), *recursive),
        }
    }

    fn undo(&self, amount: u64) -> Result<()> {
        Ok(())
    }

    fn redo(&self, amount: u64) -> Result<()> {
        Ok(())
    }

    fn inspect(&self, name: &str) -> Result<()> {
        Ok(())
    }

    fn rename(
        &self,
        name: &str,
        arguments: Option<&Vec<String>>,
        recursive: bool,
    ) -> Result<()> {
        Ok(())
    }
}
