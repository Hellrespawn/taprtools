use crate::Result;
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

/// Rename can't cross filesystems/mountpoints.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum MoveMode {
    CopyRemove,
    Rename,
}

/// Represents a single, undoable [Action].
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    Move {
        source: PathBuf,
        target: PathBuf,
        move_mode: MoveMode,
    },
    CreateDir {
        path: PathBuf,
    },
    RemoveDir {
        path: PathBuf,
    },
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
            Self::Move {
                source,
                target,
                move_mode,
            } => {
                write!(
                    f,
                    " ({}): {}\nto: {}",
                    match move_mode {
                        MoveMode::CopyRemove => "cp",
                        MoveMode::Rename => "rn",
                    },
                    source.display(),
                    target.display()
                )
            }
        }
    }
}

impl Action {
    pub fn new_move<P, Q>(source: P, target: Q, move_mode: MoveMode) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        Action::Move {
            source: PathBuf::from(source.as_ref()),
            target: PathBuf::from(target.as_ref()),
            move_mode,
        }
    }

    fn move_file<P, Q>(source: P, target: Q, move_mode: MoveMode) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        match move_mode {
            MoveMode::CopyRemove => {
                std::fs::copy(&source, &target)?;
                std::fs::remove_file(&source)?;
            }

            MoveMode::Rename => {
                std::fs::rename(&source, &target)?;
            }
        }

        Ok(())
    }

    /// Applies or "does" the action.
    pub fn apply(&self) -> Result<()> {
        match self {
            Action::Move {
                source,
                target,
                move_mode,
            } => {
                Action::move_file(source, target, *move_mode)?;

                trace!(
                    "Renamed:\n\"{}\"\n\"{}\"",
                    &source.display(),
                    &target.display()
                );
            }

            // TODO? Fail silently if dir already exists?
            Action::CreateDir { path } => {
                std::fs::create_dir(path)?;
                trace!("Created directory {}", path.display());
            }

            Action::RemoveDir { path } => {
                std::fs::remove_dir(path)?;
                trace!("Removed directory {}", path.display());
            }
        }
        Ok(())
    }

    /// Undoes the action.
    pub fn undo(&self) -> Result<()> {
        match self {
            Action::Move {
                source,
                target,
                move_mode,
            } => {
                Action::move_file(target, source, *move_mode)?;

                trace!(
                    "Undid:\n\"{}\"\n\"{}\"",
                    &target.display(),
                    &source.display(),
                );
            }

            Action::CreateDir { path } => {
                trace!("Undoing directory {}", path.display());

                std::fs::remove_dir(path)?;

                trace!("Undid directory {}", path.display());
            }

            // TODO? Fail silently if dir already exists?
            Action::RemoveDir { path } => {
                std::fs::create_dir(path)?;

                trace!("Recreated directory {}", path.display());
            }
        }
        Ok(())
    }

    /// Redoes the action. Currently only delegates to `self.apply`.
    pub fn redo(&self) -> Result<()> {
        self.apply()
    }
}
