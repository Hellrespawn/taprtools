use anyhow::Result;
use file_history::History;

use crate::cli::Filesystem;

const PP: &str = Filesystem::PREVIEW_PREFIX;

#[derive(Copy, Clone)]
pub(crate) enum UndoMode {
    Undo,
    Redo,
}

pub(crate) struct Undo;

impl Undo {
    pub(crate) fn run(
        preview: bool,
        mode: UndoMode,
        times: usize,
    ) -> Result<()> {
        let history_path = Filesystem::get_history_path()?;
        let mut history = History::load(&history_path)?;

        let amount = if preview {
            match mode {
                UndoMode::Undo => history.undo(times)?,
                UndoMode::Redo => history.redo(times)?,
            }
        } else {
            times
        };

        let action = match mode {
            UndoMode::Undo => "Undid",
            UndoMode::Redo => "Redid",
        };

        // FIXME some sort of rollback logic for undo/redo?
        history.save()?;

        println!("{PP}{action} {amount} actions.");

        Ok(())
    }
}
