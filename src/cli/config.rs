use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

pub(crate) struct Config {}

impl Config {
    pub(crate) const HISTORY_FILENAME: &'static str = "tfmttools.hist";
    pub(crate) const PREVIEW_PREFIX: &'static str = "[P] ";

    pub(crate) fn load() -> Result<Self> {
        Ok(Self {})
    }

    fn get_project_dir() -> Result<ProjectDirs> {
        ProjectDirs::from("", "", "tfmttools")
            .ok_or_else(|| anyhow!("Unable to read project directory!"))
    }

    pub(crate) fn get_history_path(&self) -> Result<PathBuf> {
        let path = Config::get_project_dir()?
            .data_dir()
            .join(Config::HISTORY_FILENAME);
        Ok(path)
    }
}
