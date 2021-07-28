use super::argparse::{Args, Subcommand};
use super::config;
use super::inspector::{Inspector, Mode};
use crate::cli::{argparse, logging};
use anyhow::{anyhow, bail, Result};
use log::info;
use std::convert::TryInto;
use std::path::PathBuf;

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
                name,
                arguments,
                recursive,
            } => self.rename(name, arguments.as_ref(), *recursive),
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
            &config::get_script(name)
                .ok_or_else(|| anyhow!("Can't find script {}", name))?,
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
        name: &str,
        arguments: Option<&Vec<String>>,
        recursive: bool,
    ) -> Result<()> {
        bail!(
            "Rename({}, {:?}, {}) is unimplemented!",
            name,
            arguments,
            recursive
        )
    }
}
