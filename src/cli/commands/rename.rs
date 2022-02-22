use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) struct Rename<'a> {
    preview: bool,
    config: &'a Config,
    history: &'a mut History,
}

impl<'a> Rename<'a> {
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

    pub(crate) fn run(
        &mut self,
        recursion_depth: usize,
        name: &str,
        arguments: &[String],
    ) -> Result<()> {
        todo!()
    }
}
