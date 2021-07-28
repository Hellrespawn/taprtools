use super::argparse::{Args, Subcommand};
use super::inspector::{Inspector, Mode};
use super::rename::{get_audiofiles, Rename};
use super::{argparse, config, logging};
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use anyhow::{bail, Result};
use log::info;
use std::convert::{TryFrom, TryInto};
use std::path::{Path, PathBuf};
use undo::Record;

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
                recursive,
            } => self.rename(
                script_name,
                &arguments
                    .iter()
                    .map(std::ops::Deref::deref)
                    .collect::<Vec<&str>>(),
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

    fn redo(&self, amount: u64) -> Result<()> {
        bail!("Redo({}) is unimplemented!", amount)
    }

    fn undo(&self, amount: u64) -> Result<()> {
        bail!("Undo({}) is unimplemented!", amount)
    }

    fn inspect(&self, name: &str, render_ast: bool) -> Result<()> {
        Inspector::inspect(
            &config::get_script(name)?,
            if render_ast { Mode::Dot } else { Mode::Long },
        )
    }

    fn rename(
        &self,
        script_name: &str,
        arguments: &[&str],
        input_folder: &Path,
        recursive: bool,
    ) -> Result<()> {
        let path = config::get_script(script_name)?;

        let program = Parser::try_from(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let depth = if recursive { 4 } else { 1 };
        let songs = get_audiofiles(input_folder, depth)?;

        let mut intp = Interpreter::new(&program, arguments, &songs)?;

        let paths: Vec<PathBuf> =
            intp.interpret()?.iter().map(PathBuf::from).collect();

        println!("Paths:\n{:#?}", paths);

        let mut record = Record::new();

        let mut processed = Vec::new();

        for (mut song, path) in songs.into_iter().zip(paths) {
            let rename = Rename::new(&path);
            record.apply(&mut song, rename)?;
            processed.push(song);
        }

        for mut song in processed {
            record.undo(&mut song).unwrap()?
        }

        Ok(())
    }
}
