use super::helpers;
use anyhow::{anyhow, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::Write;
use std::iter::Iterator;
use std::path::{Path, PathBuf};

type Stack = Vec<ActionGroup>;
pub type ActionGroup = Vec<Action>;

const HISTORY_FILENAME: &str = "tfmttools.hist";

#[derive(Default)]
pub struct History {
    dry_run: bool,
    undo_stack: Stack,
    redo_stack: Stack,
    path: Option<PathBuf>,
    changed: bool,
}

pub enum Mode {
    Undo,
    Redo,
}

impl History {
    pub fn new(dry_run: bool) -> History {
        History {
            dry_run,
            ..Default::default()
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(
        dry_run: bool,
        config_folder: &P,
    ) -> Result<History> {
        // These were selected through path.is_file(), file_name.unwrap()
        // should be safe.
        let path = helpers::search_dir(
            config_folder,
            |p| {
                debug_assert!(p.is_file());
                p.file_name().unwrap() == HISTORY_FILENAME
            },
            1,
        )
        .into_iter()
        .find(|p| p.file_name().unwrap() == HISTORY_FILENAME)
        .ok_or_else(|| {
            anyhow!("Unable to load history from {}", HISTORY_FILENAME)
        })?;

        let serialized = std::fs::read_to_string(&path)?;
        trace!(
            "Loaded history from {}:\n{}",
            path.to_string_lossy(),
            serialized
        );

        let (undo_stack, redo_stack) = serde_json::from_str(&serialized)?;

        Ok(History {
            dry_run,
            undo_stack,
            redo_stack,
            path: Some(path),
            changed: false,
        })
    }

    pub fn save_to_path<P: AsRef<Path>>(
        &self,
        config_folder: &P,
    ) -> Result<()> {
        if !self.changed {
            info!("History was unchanged.");
            return Ok(());
        }

        let path = if let Some(path) = &self.path {
            PathBuf::from(path)
        } else {
            config_folder.as_ref().join(HISTORY_FILENAME)
        };

        let serialized = serde_json::to_string_pretty(&(
            &self.undo_stack,
            &self.redo_stack,
        ))?;

        info!("Saving history to {}", path.to_string_lossy());

        trace!("\n{}", serialized);

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

    pub fn delete(&mut self) -> Result<()> {
        if !self.dry_run {
            // This function is only called after History::load_history has
            // succeeded. Unwrap should be safe.
            debug_assert!(self.path.is_some());
            std::fs::remove_file(self.path.as_ref().unwrap())?;

            self.undo_stack = Vec::new();
            self.redo_stack = Vec::new();
            self.path = None;
            self.changed = false;
        }
        Ok(())
    }

    /// Inserts action group without applying it.
    pub fn insert(&mut self, action_group: ActionGroup) -> Result<()> {
        if !self.dry_run {
            self.undo_stack.push(action_group);
            self.changed = true;
        }

        Ok(())
    }

    fn history_function(&mut self, amount: u64, mode: Mode) -> Result<()> {
        let (name, from, to) = match mode {
            Mode::Undo => ("undo", &mut self.undo_stack, &mut self.redo_stack),
            Mode::Redo => ("redo", &mut self.redo_stack, &mut self.undo_stack),
        };

        let min = std::cmp::min(amount, u64::try_from(from.len())?);

        let s = if min == 0 {
            let s = format!("There is nothing to {}.", name);
            println!("{}", s);
            info!("{}", s);
            return Ok(());
        } else if min != amount {
            format!("Warning: there are only {} actions to {}.", min, name)
        } else {
            // FIXME title-case here
            format!("{}ing {} times...", name, min)
        };

        println!("{}", s);
        info!("{}", s);

        let (func, reverse): (fn(&Action) -> Result<()>, bool) = match mode {
            Mode::Undo => (Action::undo, true),
            Mode::Redo => (Action::redo, false),
        };

        for _ in 0..min {
            // We test the amount of actions to do,
            // pop().unwrap() should be safe.
            debug_assert!(from.last().is_some());

            let action_group = from.pop().unwrap();

            if reverse {
                action_group.iter().rev().try_for_each(|a| func(a))?
            } else {
                action_group.iter().try_for_each(|a| func(a))?
            }

            if !self.dry_run {
                to.push(action_group);
            }
        }

        if !self.dry_run {
            self.changed = true;
        }

        Ok(())
    }

    pub fn undo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Mode::Undo)
    }

    pub fn redo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Mode::Redo)
    }
}

#[derive(Deserialize, Serialize)]
pub enum Action {
    Rename {
        origin: PathBuf,
        destination: PathBuf,
        dry_run: bool,
    },
    CreateDir {
        path: PathBuf,
        dry_run: bool,
    },
}

impl Action {
    pub fn apply(&self) -> Result<()> {
        match self {
            Action::CreateDir { path, dry_run } => {
                trace!("Creating directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::create_dir_all(path)?;
                }
            }

            Action::Rename {
                origin,
                destination,
                dry_run,
            } => {
                trace!(
                    "Renaming:\n\"{}\"\n\"{}\"",
                    &origin.to_string_lossy(),
                    &destination.to_string_lossy()
                );

                if !dry_run {
                    std::fs::rename(origin, destination)?
                }
            }
        }
        Ok(())
    }

    pub fn undo(&self) -> Result<()> {
        match self {
            Action::CreateDir { path, dry_run } => {
                trace!("Removing directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::remove_dir(path)?;
                }
            }

            Action::Rename {
                origin,
                destination,
                dry_run,
            } => {
                trace!(
                    "Undoing:\n\"{}\"\n\"{}\"",
                    &destination.to_string_lossy(),
                    &origin.to_string_lossy(),
                );

                if !dry_run {
                    std::fs::rename(destination, origin)?
                }
            }
        }
        Ok(())
    }

    pub fn redo(&self) -> Result<()> {
        self.apply()
    }
}
