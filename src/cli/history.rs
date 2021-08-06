use crate::helpers::{self, pp};
use anyhow::{anyhow, bail, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{self, Display};
use std::io::Write;
use std::iter::Iterator;
use std::path::{Path, PathBuf};

pub type Stack = Vec<ActionGroup>;
pub type ActionGroup = Vec<Action>;

const HISTORY_FILENAME: &str = "tfmttools.hist";

/// Stores a history of action for the purpose of undoing them.
pub struct History {
    preview: bool,
    pub done_stack: Stack,
    pub undone_stack: Stack,
    pub path: Option<PathBuf>,
    changed: bool,
}

enum Mode {
    Undo,
    Redo,
}

impl History {
    /// Create new [History]
    pub fn new(preview: bool) -> History {
        History {
            preview,
            done_stack: Stack::new(),
            undone_stack: Stack::new(),
            path: None,
            changed: false,
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(
        preview: bool,
        path: &P,
    ) -> Result<History> {
        // These were selected through path.is_file(), file_name.unwrap()
        // should be safe.
        let path = path.as_ref();

        info!("Loading history from {}.", path.display());
        let serialized = std::fs::read(&path)?;

        let (undo_stack, redo_stack) = bincode::deserialize(&serialized)?;

        Ok(History {
            preview,
            done_stack: undo_stack,
            undone_stack: redo_stack,
            path: Some(PathBuf::from(path)),
            changed: false,
        })
    }

    /// Load [History] from `config_folder`.
    pub fn load_from_config<P: AsRef<Path>>(
        preview: bool,
        config_folder: &P,
    ) -> Result<History> {
        // These were selected through path.is_file(), file_name.unwrap()
        // should be safe.
        let path = helpers::search_path(
            config_folder,
            |p| {
                debug_assert!(p.is_file());
                p.file_name().unwrap() == HISTORY_FILENAME
            },
            1,
        )
        .into_iter()
        .next()
        .ok_or_else(|| {
            anyhow!("Unable to load history from {}", HISTORY_FILENAME)
        })?;

        History::load_from_path(preview, &path)
    }

    /// Save [History] to `self.path`
    pub fn save(&self) -> Result<()> {
        if !self.changed {
            info!("History was unchanged.");
            return Ok(());
        }

        if let Some(path) = &self.path {
            self._save(&path)
        } else {
            bail!("There is not path associated with the history file!")
        }
    }

    /// Save [History] to `self.path` or `config_folder`.
    pub fn save_to_path<P: AsRef<Path>>(
        &self,
        config_folder: &P,
    ) -> Result<()> {
        if !self.changed {
            info!("History was unchanged.");
            return Ok(());
        }

        self._save(&config_folder.as_ref().join(HISTORY_FILENAME))
    }

    fn _save<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        let path = path.as_ref();

        info!("Saving history to {}", path.display());

        let serialized =
            bincode::serialize(&(&self.done_stack, &self.undone_stack))?;

        if !self.preview {
            let mut filehandle = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;

            filehandle.write_all(&serialized)?;
        }

        Ok(())
    }

    /// Deletes the history file.
    pub fn delete(&mut self) -> Result<()> {
        if !self.preview {
            // This function is only called after History::load_history has
            // succeeded. Unwrap should be safe.
            debug_assert!(self.path.is_some());
            let path = self.path.as_ref().unwrap();

            std::fs::remove_file(path)?;

            let s = format!(
                "{}Deleted history file at {}",
                pp(self.preview),
                path.display()
            );
            println!("{}", s);
            info!("{}", s);

            self.done_stack = Vec::new();
            self.undone_stack = Vec::new();
            self.path = None;
            self.changed = false;
        }
        Ok(())
    }

    /// Inserts action group without applying it.
    pub fn insert(&mut self, action_group: ActionGroup) -> Result<()> {
        if !self.preview {
            self.done_stack.push(action_group);
            self.changed = true;
        }

        Ok(())
    }

    fn history_function(&mut self, amount: u64, mode: Mode) -> Result<()> {
        let (name, from, to) = match mode {
            Mode::Undo => {
                ("undo", &mut self.done_stack, &mut self.undone_stack)
            }
            Mode::Redo => {
                ("redo", &mut self.undone_stack, &mut self.done_stack)
            }
        };

        let min = std::cmp::min(amount, u64::try_from(from.len())?);

        let s = if min == 0 {
            let s = format!("There is nothing to {}.", name);
            println!("{}", s);
            info!("{}", s);
            return Ok(());
        } else if min != amount {
            format!(
                "{}There {} only {} action{} to {}.",
                pp(self.preview),
                if min > 1 { "are" } else { "is" },
                min,
                if min > 1 { "s" } else { "" },
                name
            )
        } else {
            format!(
                "{}{}ing {} time{}:",
                pp(self.preview),
                helpers::titlecase(name),
                min,
                if min > 1 { "s" } else { "" }
            )
        };

        println!("{}", s);
        info!("{}", s);

        for i in 0..min {
            // We test the amount of actions to do,
            // pop().unwrap() should be safe.
            debug_assert!(from.last().is_some());

            let action_group = from.pop().unwrap();
            let preview = self.preview;

            let s = format!(
                "{}: {}ing {} action{}...",
                i + 1,
                helpers::titlecase(name),
                action_group.len(),
                if action_group.len() > 1 { "s" } else { "" }
            );
            print!("{}", s);
            info!("{}\n", s);

            match mode {
                Mode::Undo => action_group
                    .iter()
                    .rev()
                    .try_for_each(|a| a.undo(preview))?,

                Mode::Redo => {
                    action_group.iter().try_for_each(|a| a.redo(preview))?
                }
            }

            println!(" Done.");

            if !self.preview {
                to.push(action_group);
            }
        }

        if !self.preview {
            self.changed = true;
        }

        Ok(())
    }

    /// Undoes the last `amount` [ActionGroup]s.
    pub fn undo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Mode::Undo)
    }

    /// Redoes the last `amount` [ActionGroup]s.
    pub fn redo(&mut self, amount: u64) -> Result<()> {
        self.history_function(amount, Mode::Redo)
    }
}

/// Represents a single, undoable [Action].
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    Rename { source: PathBuf, target: PathBuf },
    CreateDir { path: PathBuf },
    RemoveDir { path: PathBuf },
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Printing a custom struct/enum seems to always include a curly brace
        // after the name, so this unwrap should be safe.
        let string = format!("{:?}", self);
        let split = string.split_once("{");

        debug_assert!(split.is_some());

        write!(f, "{}", split.unwrap().0.trim())?;
        match self {
            Self::CreateDir { path } | Self::RemoveDir { path } => {
                write!(f, ": {}", path.display())
            }
            Self::Rename { source, target } => {
                write!(f, ": {}\nto: {}", source.display(), target.display())
            }
        }
    }
}

impl Action {
    /// Applies or "does" the action.
    pub fn apply(&self, preview: bool) -> Result<()> {
        match self {
            Action::Rename { source, target } => {
                trace!(
                    "Renaming:\n\"{}\"\n\"{}\"",
                    &source.display(),
                    &target.display()
                );

                if !preview {
                    // Rename can't cross filesystems/mountpoints. This error code is correct on Windows.
                    if let Err(err) = std::fs::rename(source, target) {
                        if err.to_string().contains("os error 17") {
                            std::fs::copy(source, target)?;
                            std::fs::remove_file(source)?;
                        } else {
                            bail!(err)
                        }
                    }
                }
            }

            Action::CreateDir { path } => {
                trace!("Creating directory {}", path.display());
                if !preview {
                    std::fs::create_dir(path)?;
                }
            }

            Action::RemoveDir { path } => {
                trace!("Removing directory {}", path.display());
                if !preview {
                    std::fs::remove_dir(path)?;
                }
            }
        }
        Ok(())
    }

    /// Undoes the action.
    pub fn undo(&self, preview: bool) -> Result<()> {
        match self {
            Action::Rename { source, target } => {
                trace!(
                    "Undoing:\n\"{}\"\n\"{}\"",
                    &target.display(),
                    &source.display(),
                );

                if !preview {
                    if let Err(err) = std::fs::rename(target, source) {
                        if err.to_string().contains("os error 17") {
                            std::fs::copy(target, source)?;
                            std::fs::remove_file(target)?;
                        } else {
                            bail!(err)
                        }
                    }
                }
            }

            Action::CreateDir { path } => {
                trace!("Undoing directory {}", path.display());
                if !preview {
                    std::fs::remove_dir(path)?;
                }
            }

            Action::RemoveDir { path } => {
                trace!("Recreating directory {}", path.display());
                if !preview {
                    std::fs::create_dir(path)?;
                }
            }
        }
        Ok(())
    }

    /// Redoes the action. Currently only delegates to `self.apply`.
    pub fn redo(&self, preview: bool) -> Result<()> {
        self.apply(preview)
    }
}
