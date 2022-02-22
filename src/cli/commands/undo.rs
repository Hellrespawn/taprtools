use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) enum UndoMode {
    Undo,
    Redo,
}

pub(crate) struct Undo<'a> {
    preview: bool,
    config: &'a Config,
    history: &'a mut History,
}

impl<'a> Undo<'a> {
    pub(crate) fn new(
        preview: bool,
        config: &'a Config,
        history: &'a mut History,
    ) -> Self {
        Self {
            preview,
            config,
            history,
        }
    }

    pub(crate) fn run(&mut self, _mode: UndoMode, _times: usize) -> Result<()> {
        todo!()
    }
}
