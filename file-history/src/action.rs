use crate::Result;
use log::trace;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a single, undoable [Action].
#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
    Move { source: PathBuf, target: PathBuf },
    CreateDir(PathBuf),
    RemoveDir(PathBuf),
}

impl Action {
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
            // FIXME Use ErrorKind::CrossesDevices when it enters stable

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

    /// Redoes the action. Currently only delegates to `self.apply`.
    pub fn redo(&self) -> Result<()> {
        self.apply()
    }
}
