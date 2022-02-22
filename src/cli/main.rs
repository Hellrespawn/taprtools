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
        let mut history = History::init(&history_path)?;

        let result = match args.command {
            Command::ClearHistory { preview } => {
                ClearHistory::new(preview, &mut history).run()
            }
            Command::ListScripts => ListScripts::new(&config).run(),
            Command::InspectScript { name, render_ast } => {
                InspectScript::new(&config).run(&name, render_ast)
            }
            Command::Undo { preview, times } => {
                Undo::new(preview, &config, &mut history)
                    .run(UndoMode::Undo, times)
            }
            Command::Redo { preview, times } => {
                Undo::new(preview, &config, &mut history)
                    .run(UndoMode::Redo, times)
            }
            Command::Rename {
                preview,
                recursion_depth,
                name,
                arguments,
            } => Rename::new(preview, &config, &mut history).run(
                recursion_depth,
                &name,
                &arguments,
            ),
        };

        // FIXME Handle nested Error
        if result.is_err() {
            history.rollback()?;
        } else {
            history.save()?;
        }

        result
    };

    if let Err(err) = result {
        pretty_print_error(err)
    }

    Ok(())
}

fn pretty_print_error(error: anyhow::Error) {
    println!("{error}");
}
