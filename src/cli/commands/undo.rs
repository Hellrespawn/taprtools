use crate::cli::Config;
use anyhow::Result;
use file_history::History;

#[derive(Copy, Clone)]
pub(crate) enum UndoMode {
    Undo,
    Redo,
}

pub(crate) fn undo(
    preview: bool,
    config: &Config,
    mode: UndoMode,
    times: usize,
) -> Result<()> {
    let history_path = config.get_history_path();
    let mut history = History::load(&history_path)?;

    let amount = if preview {
        times
    } else {
        match mode {
            UndoMode::Undo => history.undo(times)?,
            UndoMode::Redo => history.redo(times)?,
        }
    };

    let action = match mode {
        UndoMode::Undo => "Undid",
        UndoMode::Redo => "Redid",
    };

    // TODO? some sort of rollback logic for undo/redo?
    history.save()?;

    let pp = if preview { Config::PREVIEW_PREFIX } else { "" };
    println!("{pp}{action} {amount} action group.");

    Ok(())
}
