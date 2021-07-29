use super::argparse::{Args, Subcommand};
use super::history::{History, Rename};
use super::inspector::{Inspector, Mode};
use super::{argparse, config, logging};
use crate::file::audiofile::get_audiofiles;
use crate::tfmt::interpreter::Interpreter;
use crate::tfmt::parser::Parser;
use anyhow::Result;
use log::info;
use std::convert::{TryFrom, TryInto};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Main tfmttools entrypoint.
pub fn main() -> Result<()> {
    _main(&std::env::args().collect::<Vec<String>>())
}

/// Public only for integration testing. Absolutely fucking hate this pub. Kill me.
pub fn _main<S: AsRef<OsStr>>(args: &[S]) -> Result<()> {
    let args = argparse::parse_args(args)?;

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;
    info!("Parsed arguments:\n{:#?}", &args);

    let mut history = History::load_history(args.dry_run, &args.config_folder)
        .unwrap_or_default();

    // TODO Pretty-print errors
    let mut p = TFMTTools {
        args: &args,
        history: &mut history,
    };
    p.main()?;

    history.save_history(&args.config_folder)?;

    Ok(())
}

struct TFMTTools<'a> {
    args: &'a Args,
    history: &'a mut History,
}

impl<'a> TFMTTools<'a> {
    fn main(&mut self) -> Result<()> {
        self.handle_command(&self.args.subcommand)
    }

    fn handle_command(&mut self, subcommand: &Subcommand) -> Result<()> {
        match subcommand {
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

    fn list_scripts(&self) -> Result<()> {
        let paths = &config::get_all_scripts(&self.args.config_folder);

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
        self.history.redo(amount)?;

        Ok(())
    }

    fn undo(&mut self, amount: u64) -> Result<()> {
        self.history.undo(amount)?;

        Ok(())
    }

    fn inspect(&self, name: &str, render_ast: bool) -> Result<()> {
        Inspector::inspect(
            &config::get_script(name, &self.args.config_folder)?,
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
        let path = config::get_script(script_name, &self.args.config_folder)?;

        let program = Parser::try_from(&path)?.parse()?;

        // TODO Get recursion depth from somewhere.
        let depth = if recursive { 4 } else { 1 };
        let songs = get_audiofiles(&input_folder, depth)?;

        let mut intp = Interpreter::new(&program, arguments, &songs)?;

        let mut paths: Vec<PathBuf> =
            intp.interpret()?.iter().map(PathBuf::from).collect();

        if let Some(prefix) = output_folder {
            let prefix = prefix.as_ref();

            if paths.iter().any(|p| p.is_absolute()) {
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

        self.history.apply(action)?;

        Ok(())
    }
}
