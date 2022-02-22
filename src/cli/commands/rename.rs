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
        _recursion_depth: usize,
        _name: &str,
        _arguments: &[String],
    ) -> Result<()> {
        todo!()
    }
}
