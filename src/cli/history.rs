#[cfg(all(feature = "bincode", feature = "serde_json"))]
compile_error!(
    "features `crate/bincode` and `crate/serde_json` are mutually exclusive"
);

#[cfg(not(any(feature = "bincode", feature = "serde_json")))]
compile_error!(
    "requires one of features `crate/bincode` or `crate/serde_json`"
);

use super::helpers;
use anyhow::{anyhow, bail, Result};
use log::{info, trace};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::io::Write;
use std::iter::Iterator;
use std::path::{Path, PathBuf};

type Stack = Vec<ActionGroup>;
/// Group of [Action]s corresponding to one execution of the program.
pub type ActionGroup = Vec<Action>;

const HISTORY_FILENAME: &str = "tfmttools.hist";

// TODO Viewer for bincode histfile
// FIXME something wrong with undoing when there's nothing to undo.

/// Stores a history of action for the purpose of undoing them.
#[derive(Default)]
pub struct History {
    dry_run: bool,
    undo_stack: Stack,
    redo_stack: Stack,
    pub path: Option<PathBuf>,
    changed: bool,
}

enum Mode {
    Undo,
    Redo,
}

impl History {
    /// Create new [History]
    pub fn new(dry_run: bool) -> History {
        History {
            dry_run,
            ..Default::default()
        }
    }

    #[cfg(feature = "bincode")]
    fn deserialize(ser: &[u8]) -> Result<(Stack, Stack)> {
        Ok(bincode::deserialize(ser)?)
    }

    #[cfg(feature = "serde_json")]
    fn deserialize(ser: &[u8]) -> Result<(Stack, Stack)> {
        Ok(serde_json::from_slice(ser)?)
    }

    /// Load [History] from `config_folder`.
    pub fn load_from_path<P: AsRef<Path>>(
        dry_run: bool,
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

        info!("Loading history from {}.", path.to_string_lossy());
        let serialized = std::fs::read(&path)?;

        let (undo_stack, redo_stack) = History::deserialize(&serialized)?;

        Ok(History {
            dry_run,
            undo_stack,
            redo_stack,
            path: Some(path),
            changed: false,
        })
    }

    #[cfg(feature = "bincode")]
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(bincode::serialize(&(&self.undo_stack, &self.redo_stack))?)
    }

    #[cfg(feature = "serde_json")]
    fn serialize(&self) -> Result<Vec<u8>> {
        trace!(
            "{}",
            serde_json::to_string_pretty(&(
                &self.undo_stack,
                &self.redo_stack,
            ))?
        );
        Ok(serde_json::to_vec_pretty(&(
            &self.undo_stack,
            &self.redo_stack,
        ))?)
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

        info!("Saving history to {}", path.to_string_lossy());

        let serialized = self.serialize()?;

        if !self.dry_run {
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
        if !self.dry_run {
            // This function is only called after History::load_history has
            // succeeded. Unwrap should be safe.
            debug_assert!(self.path.is_some());
            let path = self.path.as_ref().unwrap();

            std::fs::remove_file(path)?;

            let s =
                format!("Deleted history file at {}", path.to_string_lossy());
            println!("{}", s);
            info!("{}", s);

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
            format!(
                "There {} only {} action{} to {}.",
                if min > 1 { "are" } else { "is" },
                min,
                if min > 1 { "s" } else { "" },
                name
            )
        } else {
            format!(
                "{}ing {} time{}:",
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
            let dry_run = self.dry_run;

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
                    .try_for_each(|a| a.undo(dry_run))?,

                Mode::Redo => {
                    action_group.iter().try_for_each(|a| a.redo(dry_run))?
                }
            }

            println!(" Done.");

            if !self.dry_run {
                to.push(action_group);
            }
        }

        if !self.dry_run {
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
#[derive(Deserialize, Serialize)]
pub enum Action {
    Rename { source: PathBuf, target: PathBuf },
    CreateDir { path: PathBuf },
    RemoveDir { path: PathBuf },
}

impl Action {
    /// Applies or "does" the action.
    pub fn apply(&self, dry_run: bool) -> Result<()> {
        match self {
            Action::Rename { source, target } => {
                trace!(
                    "Renaming:\n\"{}\"\n\"{}\"",
                    &source.to_string_lossy(),
                    &target.to_string_lossy()
                );

                if !dry_run {
                    std::fs::rename(source, target)?
                }
            }

            Action::CreateDir { path } => {
                trace!("Creating directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::create_dir(path)?;
                }
            }

            Action::RemoveDir { path } => {
                trace!("Removing directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::remove_dir(path)?;
                }
            }
        }
        Ok(())
    }

    /// Undoes the action.
    pub fn undo(&self, dry_run: bool) -> Result<()> {
        match self {
            Action::Rename { source, target } => {
                trace!(
                    "Undoing:\n\"{}\"\n\"{}\"",
                    &target.to_string_lossy(),
                    &source.to_string_lossy(),
                );

                if !dry_run {
                    std::fs::rename(target, source)?
                }
            }

            Action::CreateDir { path } => {
                trace!("Undoing directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::remove_dir(path)?;
                }
            }

            Action::RemoveDir { path } => {
                trace!("Recreating directory {}", path.to_string_lossy());
                if !dry_run {
                    std::fs::create_dir(path)?;
                }
            }
        }
        Ok(())
    }

    /// Redoes the action. Currently only delegates to `self.apply`.
    pub fn redo(&self, dry_run: bool) -> Result<()> {
        self.apply(dry_run)
    }
}
