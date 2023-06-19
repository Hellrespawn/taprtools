use crate::cli::args::Command;
use crate::cli::commands::{self, UndoMode};
use crate::cli::{ui, Args, Config};
use anyhow::Result;

/// Main entrypoint for taprtools
pub fn main(preview_override: bool) -> Result<()> {
    let args = crate::cli::args::parse_args(preview_override);

    if let Err(err) = select_command(args) {
        ui::print_error(&err);
    }

    Ok(())
}

fn select_command(args: Args) -> Result<()> {
    let config = if let Some(path) = &args.config {
        Config::new(path)?
    } else {
        Config::new(&Config::default_path()?)?
    };

    match args.command {
        Command::ClearHistory { preview } => {
            commands::clear_history(preview, &config)
        }
        Command::ListScripts => commands::list_scripts(&config),
        Command::Undo { preview, times } => {
            commands::undo(preview, &config, UndoMode::Undo, times)
        }
        Command::Redo { preview, times } => {
            commands::undo(preview, &config, UndoMode::Redo, times)
        }
        Command::Rename {
            preview,
            recurse,
            name,
            arguments,
        } => commands::rename(preview, &config, recurse, &name, &arguments),

        Command::Seed { preview, force } => {
            commands::seed(preview, force, &config)
        }
    }
}
