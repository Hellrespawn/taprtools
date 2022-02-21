use anyhow::{anyhow, Result};
use directories::ProjectDirs;

pub(crate) const DEFAULT_HISTORY_FILENAME: &str = "tfmttools.hist";
pub(crate) const DEFAULT_PREVIEW_AMOUNT: usize = 8;
pub(crate) const DEFAULT_RECURSION_DEPTH: usize = 4;
pub(crate) const PREVIEW_PREFIX: &str = "[P] ";

pub(crate) fn get_project_dir() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "tfmttools")
        .ok_or_else(|| anyhow!("Unable to read project directory!"))
}
