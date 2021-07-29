use super::config;
use anyhow::{anyhow, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::Write;
use std::path::{Path, PathBuf};

const HISTORY_FILENAME: &str = "tfmttools.hist";

#[derive(Default, Deserialize, Serialize)]
pub struct History {
    pub undo_stack: Vec<Vec<Rename>>,
    pub redo_stack: Vec<Vec<Rename>>,
    pub path: Option<PathBuf>,
    pub dry_run: bool,
}

pub enum Action {
    Undo,
    Redo,
}

impl History {
    pub fn new() -> History {
        Default::default()
    }

    pub fn load_history(dry_run: bool) -> Result<History> {
        // These were selected through path.is_file(), file_name.unwrap()
        // should be safe.
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

        let (undo_actions, redo_actions) = serde_json::from_str(&serialized)?;

        Ok(History {
            undo_stack: undo_actions,
            redo_stack: redo_actions,
            path: Some(path),
            dry_run,
        })
    }

    pub fn save_history(&self) -> Result<()> {
        let path = if let Some(path) = &self.path {
            PathBuf::from(path)
        } else {
            PathBuf::from(
                config::get_config_dirs().iter().next().ok_or_else(|| {
                    anyhow!("Can't find any valid config dirs!")
                })?,
            )
        }
        .join(HISTORY_FILENAME);

        let serialized = serde_json::to_string_pretty(&(
            &self.undo_stack,
            &self.redo_stack,
        ))?;

        trace!(
            "Saving history to {}:\n{}",
            path.to_string_lossy(),
            serialized
        );

        if !self.dry_run {
            let mut filehandle = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;

            write!(filehandle, "{}", serialized)?;
        }

        Ok(())
    }

    pub fn apply(&mut self, action: Vec<Rename>) -> Result<()> {
        for rename in &action {
            rename.apply(self.dry_run)?;
        }

        if !self.dry_run {
            self.undo_stack.push(action);
        }

        Ok(())
    }

    fn history_function(&mut self, amount: u64, action: Action) -> Result<()> {
        let (name, from, to) = match action {
            Action::Undo => {
                ("undo", &mut self.undo_stack, &mut self.redo_stack)
            }
            Action::Redo => {
                ("redo", &mut self.redo_stack, &mut self.undo_stack)
            }
        };

        let method: fn(&Rename, bool) -> Result<()> = match action {
            Action::Undo => Rename::undo,
            Action::Redo => Rename::redo,
        };

        let min = std::cmp::min(amount, u64::try_from(from.len())?);

        if min != amount {
            println!("Warning: there are only {} actions to {}.", min, name)
        }

        info!("{}ing {} times...", name, min);

        for _ in 0..min {
            // We test in the above line, pop().unwrap() should be safe.
            let action = from.pop().unwrap();

            for rename in &action {
                method(rename, self.dry_run)?
                //rename.undo(dry_run)?
            }

            if !self.dry_run {
                to.push(action);
            }
        }

        Ok(())
    }

    pub fn undo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Action::Undo)
    }

    pub fn redo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Action::Redo)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Rename {
    destination: PathBuf,
    origin: PathBuf,
}

impl Rename {
    pub fn new<P: AsRef<Path>>(destination: &P, origin: &P) -> Rename {
        Rename {
            destination: PathBuf::from(destination.as_ref()),
            origin: PathBuf::from(origin.as_ref()),
        }
    }
}

impl Rename {
    pub fn apply(&self, dry_run: bool) -> Result<()> {
        trace!(
            "Creating directory: \"{}\"",
            self.destination
                .parent()
                .ok_or_else(|| anyhow!(
                    "AudioFile doesn't have a parent directory!"
                ))?
                .to_string_lossy()
        );
        if !dry_run {
            std::fs::create_dir_all(self.destination.parent().ok_or_else(
                || anyhow!("AudioFile doesn't have a parent directory!"),
            )?)?;
        }

        trace!(
            "Renaming:\n\"{}\"\n\"{}\"",
            &self.origin.to_string_lossy(),
            &self.destination.to_string_lossy()
        );
        if !dry_run {
            std::fs::rename(&self.origin, &self.destination)?;
        }

        Ok(())
    }

    pub fn undo(&self, dry_run: bool) -> Result<()> {
        trace!(
            "Creating directory: \"{}\"",
            self.origin
                .parent()
                .ok_or_else(|| anyhow!(
                    "AudioFile doesn't have a parent directory!"
                ))?
                .to_string_lossy()
        );
        if !dry_run {
            std::fs::create_dir_all(self.origin.parent().ok_or_else(
                || anyhow!("AudioFile doesn't have a parent directory!"),
            )?)?;
        }
        trace!(
            "Undoing:\n\"{}\"\n\"{}\"",
            &self.destination.to_string_lossy(),
            &self.origin.to_string_lossy()
        );
        if !dry_run {
            std::fs::rename(&self.destination, &self.origin)?;
        }
        Ok(())
    }

    pub fn redo(&self, dry_run: bool) -> Result<()> {
        self.apply(dry_run)
    }
}
