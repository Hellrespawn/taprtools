use crate::cli::Config;
use anyhow::Result;
use file_history::History;

const PP: &str = Config::PREVIEW_PREFIX;

pub(crate) struct ClearHistory<'a> {
    preview: bool,
    history: &'a mut History,
}

impl<'a> ClearHistory<'a> {
    pub(crate) fn new(preview: bool, history: &'a mut History) -> Self {
        Self { preview, history }
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        if self.preview {
            self.history.clear()?;
        }

        println!("{PP}Cleared history.");
        Ok(())
    }
}
