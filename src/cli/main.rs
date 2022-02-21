use crate::cli::args::Command;
use anyhow::Result;
use file_history::History;

use crate::cli::commands::{
    ClearHistory, InspectScript, ListScripts, Rename, Undo, UndoMode,
};
use crate::cli::Config;

/// Main entrypoint for tfmttools
pub fn main(preview_override: bool) -> Result<()> {
    let result: Result<()> = {
        let args = crate::cli::args::parse_args(preview_override);

        let config = Config::load()?;

        let history_path = config.get_history_path()?;
        let history = History::init(&history_path)?;

        match args.command {
            Command::ClearHistory { preview } => {
                ClearHistory::new(preview, config, history).run()
            }
            Command::ListScripts => ListScripts::new(config).run(),
            Command::InspectScript { name } => {
                InspectScript::new(config).run(&name)
            }
            Command::Undo { preview, times } => {
                Undo::new(preview, config, history).run(UndoMode::Undo, times)
            }
            Command::Redo { preview, times } => {
                Undo::new(preview, config, history).run(UndoMode::Redo, times)
            }
            Command::Rename {
                preview,
                recursion_depth,
                name,
                arguments,
            } => Rename::new(preview, config, history).run(
                recursion_depth,
                &name,
                &arguments,
            ),
        }?;

        Ok(())
    };

    if let Err(err) = result {
        pretty_print_error(err)
    }

    Ok(())
}

fn pretty_print_error(error: anyhow::Error) {
    println!("{error}");
}
