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
    let mut history = History::load(config.path(), Config::HISTORY_NAME)?;

    let mode_string = match mode {
        UndoMode::Undo => "Undid",
        UndoMode::Redo => "Redid",
    };

    if preview {
        let pp = Config::PREVIEW_PREFIX;
        println!("{pp}{mode_string} {times} renames.");
    } else {
        let action_counts = match mode {
            UndoMode::Undo => history.undo(times)?,
            UndoMode::Redo => history.redo(times)?,
        };

        // TODO? some sort of rollback logic for errors during undo/redo?
        history.save()?;

        println!("{} {} renames:", mode_string, action_counts.len());
        for (i, action_count) in action_counts.into_iter().enumerate() {
            println!(
                "{}: {} moves, {} dirs created, {} dirs removed",
                i + 1,
                action_count.mv,
                action_count.mkdir,
                action_count.rmdir
            );
        }
    }

    Ok(())
}
