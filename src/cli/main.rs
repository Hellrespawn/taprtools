use crate::cli::args::Command;
use crate::cli::commands::*;
use crate::cli::{Args, Config};
use anyhow::Result;

/// Main entrypoint for tfmttools
pub fn main(preview_override: bool) -> Result<()> {
    let args = crate::cli::args::parse_args(preview_override);

    with_custom_args(args)
}

/// Runs tfmttools with custom arguments.
pub fn with_custom_args(args: Args) -> Result<()> {
    let config = if let Some(path) = args.config_folder {
        Config::new(&path)?
    } else {
        Config::default()?
    };

    match args.command {
        Command::ClearHistory { preview } => clear_history(preview, &config),
        Command::ListScripts => list_scripts(&config),
        Command::InspectScript { name, render_ast } => {
            inspect_script(&config, &name, render_ast)
        }
        Command::Undo { preview, times } => {
            undo(preview, &config, UndoMode::Undo, times)
        }
        Command::Redo { preview, times } => {
            undo(preview, &config, UndoMode::Redo, times)
        }
        Command::Rename {
            preview,
            recursion_depth,
            name,
            arguments,
        } => rename(preview, &config, recursion_depth, &name, &arguments),
    }
}
