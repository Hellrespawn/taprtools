use anyhow::{bail, Result};
use std::path::PathBuf;

pub(crate) struct Config {}

impl Config {
    pub(crate) const HISTORY_FILENAME: &'static str = "tfmttools.hist";
    pub(crate) const PREVIEW_PREFIX: &'static str = "[P] ";
    pub(crate) const SCRIPT_EXTENSION: &'static str = "tfmt";

    pub(crate) fn load() -> Result<Self> {
        Ok(Self {})
    }

    fn get_project_dir() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            let project_dir = home.join(format!(".{}", env!("CARGO_PKG_NAME")));
            if !project_dir.exists() {
                std::fs::create_dir(&project_dir)?;
            } else if !project_dir.is_dir() {
                bail!("Unable to create project directory!")
            }

            Ok(project_dir)
        } else {
            bail!("Unable to read home directory!")
        }
    }

    pub(crate) fn get_history_path(&self) -> Result<PathBuf> {
        let path = Config::get_project_dir()?.join(Config::HISTORY_FILENAME);
        Ok(path)
    }

    pub(crate) fn get_script_paths(&self) -> Result<Vec<PathBuf>> {
        // FIXME also get scripts in cwd?
        let path = Config::get_project_dir()?;

        let paths = crate::helpers::search_path(&path, 0, |p| {
            p.extension()
                .map(|s| s == Config::SCRIPT_EXTENSION)
                .unwrap_or(false)
        });

        Ok(paths)
    }
}
