use crate::cli::args::Command;
use anyhow::Result;

use crate::cli::commands::{
    ClearHistory, InspectScript, ListScripts, Rename, Undo, UndoMode,
};

/// Main entrypoint for tfmttools
pub fn main(preview_override: bool) -> Result<()> {
    let args = crate::cli::args::parse_args(preview_override);

    match args.command {
        Command::ClearHistory { preview } => ClearHistory::run(preview),
        Command::ListScripts => ListScripts::run(),
        Command::InspectScript { name, render_ast } => {
            InspectScript::run(&name, render_ast)
        }
        Command::Undo { preview, times } => {
            Undo::run(preview, UndoMode::Undo, times)
        }
        Command::Redo { preview, times } => {
            Undo::run(preview, UndoMode::Redo, times)
        }
        Command::Rename {
            preview,
            recursion_depth,
            name,
            arguments,
        } => Rename::run(preview, recursion_depth, &name, &arguments),
    }
}
