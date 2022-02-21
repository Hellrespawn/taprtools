use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) enum UndoMode {
    Undo,
    Redo,
}

pub(crate) struct Undo {
    preview: bool,
    config: Config,
    history: History,
}

impl Undo {
    pub(crate) fn new(preview: bool, config: Config, history: History) -> Self {
        Self {
            preview,
            config,
            history,
        }
    }

    pub(crate) fn run(&self, mode: UndoMode, times: usize) -> Result<()> {
        todo!()
    }
}
