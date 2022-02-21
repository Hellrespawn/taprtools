use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) struct ClearHistory {
    preview: bool,
    config: Config,
    history: History,
}

impl ClearHistory {
    pub(crate) fn new(preview: bool, config: Config, history: History) -> Self {
        Self {
            preview,
            config,
            history,
        }
    }

    pub(crate) fn run(&self) -> Result<()> {
        todo!()
    }
}
