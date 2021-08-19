use crate::{Action, HistoryError, Result};
use log::info;
use std::convert::TryFrom;
use std::io::Write;
use std::path::{Path, PathBuf};

pub type Stack = Vec<ActionGroup>;
pub type ActionGroup = Vec<Action>;

/// Stores a history of action for the purpose of undoing them.

#[derive(Default)]
pub struct History {
    pub done_stack: Stack,
    pub undone_stack: Stack,
    pub path: Option<PathBuf>,
    changed: bool,
    quiet: bool,
}

enum Mode {
    Undo,
    Redo,
}

impl History {
    /// Create new [History]
    pub fn new(quiet: bool) -> History {
        History {
            done_stack: Stack::new(),
            undone_stack: Stack::new(),
            path: None,
            changed: false,
            quiet,
        }
    }

    pub fn load_file<P: AsRef<Path>>(path: &P, quiet: bool) -> Result<History> {
        let path = path.as_ref();

        info!("Loading history from {}.", path.display());
        let serialized = std::fs::read(&path)?;

        let (undo_stack, redo_stack) = bincode::deserialize(&serialized)?;

        Ok(History {
            done_stack: undo_stack,
            undone_stack: redo_stack,
            path: Some(PathBuf::from(path)),
            changed: false,
            quiet,
        })
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
            Err(HistoryError::NoPath)
        }
    }

    /// Save [History] to `self.path` or `config_folder`.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        if !self.changed {
            info!("History was unchanged.");
            return Ok(());
        }

        self._save(path)
    }

    fn _save<P: AsRef<Path>>(&self, path: &P) -> Result<()> {
        let path = path.as_ref();

        info!("Saving history to {}", path.display());

        let serialized =
            bincode::serialize(&(&self.done_stack, &self.undone_stack))?;

        let mut filehandle = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;

        filehandle.write_all(&serialized)?;

        Ok(())
    }

    /// Deletes the history file.
    pub fn delete(&mut self) -> Result<()> {
        self.done_stack = Vec::new();
        self.undone_stack = Vec::new();
        self.path = None;
        self.changed = false;

        let string = if let Some(path) = &self.path {
            std::fs::remove_file(&path)?;
            format!("Deleted history file at {}.", path.display())
        } else {
            "Deleted history file.".to_string()
        };

        info!("{}", string);

        if !self.quiet {
            println!("{}", string)
        }

        Ok(())
    }

    /// Inserts action group without applying it.
    pub fn insert(&mut self, action_group: ActionGroup) -> Result<()> {
        self.done_stack.push(action_group);
        self.changed = true;

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

        let string = if min == 0 {
            format!("There is nothing to {}.", name)
        } else if min != amount {
            format!(
                "There {} only {} action{} to {}:",
                if min > 1 { "are" } else { "is" },
                min,
                if min > 1 { "s" } else { "" },
                name
            )
        } else {
            format!(
                "{}ing {} time{}:",
                crate::titlecase(name),
                min,
                if min > 1 { "s" } else { "" }
            )
        };

        info!("{}", string);
        if !self.quiet {
            println!("{}", string)
        }

        for i in 0..min {
            // We test the amount of actions to do,
            // pop().unwrap() should be safe.
            debug_assert!(from.last().is_some());

            let action_group = from.pop().unwrap();

            let string = format!(
                "{}: {}ing {} action{}...",
                i + 1,
                crate::titlecase(name),
                action_group.len(),
                if action_group.len() > 1 { "s" } else { "" }
            );

            info!("{}", string);
            if !self.quiet {
                print!("{}", string)
            }

            match mode {
                Mode::Undo => {
                    action_group.iter().rev().try_for_each(|a| a.undo())?
                }

                Mode::Redo => action_group.iter().try_for_each(|a| a.redo())?,
            }

            if !self.quiet {
                println!(" Done.")
            }

            to.push(action_group);

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
