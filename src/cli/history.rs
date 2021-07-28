use super::config;
use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use log::trace;

const HISTORY_FILENAME: &str = "tfmttools.hist";

#[derive(Default, Deserialize, Serialize)]
pub struct History {
    pub actions: Vec<Rename>,
    pub path: Option<PathBuf>,
}

impl History {
    pub fn new() -> History {
        Default::default()
    }

    pub fn load_history() -> Result<History> {
        // These were selected through path.is_file(), unwrap should be safe.
        let path = config::get_config_dirs()
            .iter()
            .map(|d| config::search_dir_for_filename(d, HISTORY_FILENAME))
            .flatten()
            .find(|p| p.file_name().unwrap() == HISTORY_FILENAME)
            .ok_or_else(|| {
                anyhow!("Unable to load history from {}", HISTORY_FILENAME)
            })?;

        let serialized = std::fs::read_to_string(&path)?;
        trace!("Loaded history:\n{}", serialized);

        Ok(History::new())

        // Ok(History {
        //     record: serde_json::from_str(&serialized)?,
        //     path: Some(path),
        // })
    }

    pub fn save_history(&self) -> Result<()> {
        let mut path = if let Some(path) = &self.path {
            PathBuf::from(path)
        } else {
            PathBuf::from(
                config::get_config_dirs().iter().next().ok_or_else(|| {
                    anyhow!("Can't find any valid config dirs!")
                })?,
            )
        };

        path.push(HISTORY_FILENAME);

        //let serialized = serde_json::to_string_pretty(&self.record)?;
        let serialized = String::new();
        trace!("Saving history to {}:\n{}", path.to_string_lossy(), serialized);

        let mut filehandle = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        write!(filehandle, "{}", serialized)?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Rename {}
