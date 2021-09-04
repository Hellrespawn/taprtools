use crate::Result;
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

/// Represents a single, undoable [Action].
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    Move { source: PathBuf, target: PathBuf },
    CreateDir(PathBuf),
    RemoveDir(PathBuf),
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
            Self::CreateDir(path) | Self::RemoveDir(path) => {
                write!(f, ": {}", path.display())
            }
            Self::Move { source, target } => {
                write!(f, ": {}\nto: {}", source.display(), target.display())
            }
        }
    }
}

impl Action {
    pub fn move_file<P, Q>(source: P, target: Q) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        Action::Move {
            source: PathBuf::from(source.as_ref()),
            target: PathBuf::from(target.as_ref()),
        }
    }

    fn copy_or_move_file<P, Q>(source: P, target: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        if let Err(err) = std::fs::rename(&source, &target) {
            // Can't rename across filesystem boundaries. Checks for
            // the appropriate error and changes the mode henceforth.
            // Error codes are correct on Windows 10 20H2 and Arch
            // Linux.

            #[cfg(windows)]
            let expected_error = err.to_string().contains("os error 17");

            #[cfg(unix)]
            let expected_error = err.to_string().contains("os error 18");

            if !expected_error {
                return Err(err.into());
            } else {
                std::fs::copy(&source, &target)?;
                std::fs::remove_file(&source)?;
            }
        }

        Ok(())
    }

    /// Applies or "does" the action.
    pub fn apply(&self) -> Result<()> {
        match self {
            Action::Move { source, target } => {
                Action::copy_or_move_file(source, target)?;

                trace!(
                    "Renamed:\n\"{}\"\n\"{}\"",
                    &source.display(),
                    &target.display()
                );
            }

            // TODO? Fail silently if dir already exists?
            Action::CreateDir(path) => {
                std::fs::create_dir(path)?;
                trace!("Created directory {}", path.display());
            }

            Action::RemoveDir(path) => {
                std::fs::remove_dir(path)?;
                trace!("Removed directory {}", path.display());
            }
        }
        Ok(())
    }

    /// Undoes the action.
    pub fn undo(&self) -> Result<()> {
        match self {
            Action::Move { source, target } => {
                Action::copy_or_move_file(target, source)?;

                trace!(
                    "Undid:\n\"{}\"\n\"{}\"",
                    &target.display(),
                    &source.display(),
                );
            }

            Action::CreateDir(path) => {
                trace!("Undoing directory {}", path.display());

                std::fs::remove_dir(path)?;

                trace!("Undid directory {}", path.display());
            }

            // TODO? Fail silently if dir already exists?
            Action::RemoveDir(path) => {
                std::fs::create_dir(path)?;

                trace!("Recreated directory {}", path.display());
            }
        }
        Ok(())
    }

    /// Alias for `self.apply`.
    pub fn r#do(&self) -> Result<()> {
        self.apply()
    }

    /// Redoes the action. Currently only delegates to `self.apply`.
    pub fn redo(&self) -> Result<()> {
        self.apply()
    }
}
