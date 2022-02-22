use crate::cli::Filesystem;
use anyhow::Result;
use file_history::History;

const PP: &str = Filesystem::PREVIEW_PREFIX;

pub(crate) struct ClearHistory;

impl ClearHistory {
    pub(crate) fn run(preview: bool) -> Result<()> {
        if preview {
            let path = Filesystem::get_history_path()?;
            let mut history = History::load(&path)?;
            history.clear()?;
        }

        println!("{PP}Cleared history.");
        Ok(())
    }
}
