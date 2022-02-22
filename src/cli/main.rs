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
        let mut history: Option<History> = None;

        let result =
            match args.command {
                Command::ClearHistory { preview } => {
                    history = Some(History::init(&history_path)?);
                    ClearHistory::new(preview, history.as_mut().unwrap()).run()
                }
                Command::ListScripts => ListScripts::new(&config).run(),
                Command::InspectScript { name, render_ast } => {
                    InspectScript::new(&config).run(&name, render_ast)
                }
                Command::Undo { preview, times } => {
                    history = Some(History::init(&history_path)?);
                    Undo::new(preview, &config, history.as_mut().unwrap())
                        .run(UndoMode::Undo, times)
                }
                Command::Redo { preview, times } => {
                    history = Some(History::init(&history_path)?);
                    Undo::new(preview, &config, history.as_mut().unwrap())
                        .run(UndoMode::Redo, times)
                }
                Command::Rename {
                    preview,
                    recursion_depth,
                    name,
                    arguments,
                } => {
                    history = Some(History::init(&history_path)?);
                    Rename::new(preview, &config, history.as_mut().unwrap())
                        .run(recursion_depth, &name, &arguments)
                }
            };

        // FIXME Handle nested Error
        if let Some(mut history) = history {
            if result.is_err() {
                history.rollback()?;
            } else {
                history.save()?;
            }
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
