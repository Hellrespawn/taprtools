use crate::Result;
use log::trace;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Action is responsible for doing and undoing filesystem operations
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Action {
    /// Represents the moving of a file.
    Move {
        /// Source path
        source: PathBuf,
        /// Target path
        target: PathBuf,
    },
    /// Represents the creating of a directory
    MakeDir(PathBuf),
    /// Represents the deletion of a directory
    RemoveDir(PathBuf),
}

impl Action {
    /// Applies the action
    pub(crate) fn apply(&self) -> Result<()> {
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
            Action::MakeDir(path) => {
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

            Action::MakeDir(path) => {
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

    fn copy_or_move_file<P, Q>(source: P, target: Q) -> Result<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        if let Err(err) = std::fs::rename(&source, &target) {
            // Can't rename across filesystem boundaries. Checks for
            // the appropriate error and copies/deletes instead.
            // Error codes are correct on Windows 10 20H2 and Arch
            // Linux.
            // FIXME Use ErrorKind::CrossesDevices when it enters stable

            if let Some(error_code) = err.raw_os_error() {
                #[cfg(windows)]
                let expected_error_code = 17;

                #[cfg(unix)]
                let expected_error_code = 18;

                return if expected_error_code == error_code {
                    std::fs::copy(&source, &target)?;
                    std::fs::remove_file(&source)?;
                    Ok(())
                } else {
                    Err(err.into())
                };
            }

            return Err(err.into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
