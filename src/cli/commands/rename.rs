use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) struct Rename {
    preview: bool,
    config: Config,
    history: History,
}

impl Rename {
    pub(crate) fn new(preview: bool, config: Config, history: History) -> Self {
        Self {
            preview,
            config,
            history,
        }
    }

    pub(crate) fn run(
        &self,
        recursion_depth: usize,
        name: &str,
        arguments: &[String],
    ) -> Result<()> {
        todo!()
    }
}
