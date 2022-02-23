use crate::cli::Config;
use anyhow::Result;
use file_history::History;

pub(crate) fn clear_history(preview: bool, config: &Config) -> Result<()> {
    if preview {
        let path = config.get_history_path();
        let mut history = History::load(&path)?;
        history.clear()?;
    }

    let pp = if preview { Config::PREVIEW_PREFIX } else { "" };
    println!("{pp}Cleared history.");
    Ok(())
}
