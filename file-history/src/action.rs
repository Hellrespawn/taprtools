use crate::{HistoryError, Result};
use log::trace;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Action is responsible for doing and undoing filesystem operations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = "  ";
        match self {
            Action::Move { source, target } => write!(
                f,
                "Action::Move {{\n{indent}source: \"{}\",\n{indent}target: \"{}\"\n}}",
                source.display(),
                target.display()
            )?,
            Action::MakeDir(path) => {
                write!(
                    f, "Action::MakeDir(\n{indent}\"{}\"\n)", path.display()
                )?;
            }
            Action::RemoveDir(path) => {
                write!(
                    f, "Action::RemoveDir(\n{indent}\"{}\"\n)", path.display()
                )?;
            }
        }

        Ok(())
    }
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
                std::fs::create_dir(path).map_err(|err| {
                    HistoryError::new(&format!(
                        "Error while creating {}:\n{}",
                        path.display(),
                        err
                    ))
                })?;
                trace!("Created directory {}", path.display());
            }

            Action::RemoveDir(path) => {
                std::fs::remove_dir(path).map_err(|err| {
                    HistoryError::new(&format!(
                        "Error while removing {}:\n{}",
                        path.display(),
                        err
                    ))
                })?;
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
                std::fs::remove_dir(path).map_err(|err| {
                    HistoryError::new(&format!(
                        "Error while undoing creation of {}:\n{}",
                        path.display(),
                        err
                    ))
                })?;

                trace!("Undid directory {}", path.display());
            }

            // TODO? Fail silently if dir already exists?
            Action::RemoveDir(path) => {
                std::fs::create_dir(path).map_err(|err| {
                    HistoryError::new(&format!(
                        "Error while undoing removal of {}:\n{}",
                        path.display(),
                        err
                    ))
                })?;

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

                if expected_error_code == error_code {
                    std::fs::copy(&source, &target)?;
                    std::fs::remove_file(&source)?;
                    return Ok(());
                };
            }

            Err(HistoryError::new(&format!(
                "Error while renaming:\nsource: {}\ntarget: {}\n{}",
                source.as_ref().display(),
                target.as_ref().display(),
                err,
            )))
        } else {
            Ok(())
        }
    }

    /// Gets source and target from this action.
    ///
    /// # Panics
    ///
    /// This function panics if this action is not `Action::Move`
    pub fn get_src_tgt_unchecked(&self) -> (&Path, &Path) {
        if let Action::Move { source, target } = self {
            (source, target)
        } else {
            panic!("Current Action is not Action::Move!")
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO Write test for applying twice
    // TODO Write test for undoing before applying
    // TODO Write test for undoing file that's been moved
    use super::*;
    use tempfile::{Builder, NamedTempFile, TempDir};

    static PREFIX: &str = "rust-file-history-action-";

    fn get_temporary_dir() -> Result<TempDir> {
        let dir = Builder::new().prefix(PREFIX).tempdir()?;
        Ok(dir)
    }

    fn get_temporary_file(path: &Path) -> Result<NamedTempFile> {
        let tempfile = NamedTempFile::new_in(path)?;
        Ok(tempfile)
    }

    #[test]
    fn test_make_dir() -> Result<()> {
        let dir = get_temporary_dir()?;
        let path = dir.path().join("test");
        let mkdir = Action::MakeDir(path.to_path_buf());

        // Before: doesn't exist
        assert!(!path.is_dir());

        mkdir.apply()?;

        // Applied: exists
        assert!(path.is_dir());

        mkdir.undo()?;

        // Undone: doesn't exist
        assert!(!path.is_dir());

        Ok(())
    }

    #[test]
    fn test_remove_dir() -> Result<()> {
        let dir = get_temporary_dir()?;

        // Before: exists
        assert!(dir.path().is_dir());

        let rmdir = Action::RemoveDir(dir.path().to_path_buf());

        rmdir.apply()?;

        // Applied: doesn't exist
        assert!(!dir.path().is_dir());

        rmdir.undo()?;

        // Undone: exists
        assert!(dir.path().is_dir());

        Ok(())
    }

    #[test]
    fn test_move() -> Result<()> {
        let dir = get_temporary_dir()?;
        let file = get_temporary_file(dir.path())?;

        let source = file.path().to_path_buf();
        let target = file.path().with_file_name("test").to_path_buf();

        // Before: source exists, target doesn't
        assert!(source.is_file());
        assert!(!target.is_file());

        let mv = Action::Move {
            source: source.to_path_buf(),
            target: target.to_path_buf(),
        };

        mv.apply()?;

        // Applied: source doesn't, target exists
        assert!(!source.is_file());
        assert!(target.is_file());

        mv.undo()?;

        // Undone: source exists, target doesn't
        assert!(source.is_file());
        assert!(!target.is_file());

        Ok(())
    }
}
