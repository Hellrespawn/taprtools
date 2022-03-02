use crate::Result;
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
            // UPSTREAM Use ErrorKind::CrossesDevices when it enters stable

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

            Err(err.into())
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
    use anyhow::Result;
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use predicates::prelude::*;

    #[test]
    fn test_make_dir() -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.child("test");

        let action = Action::MakeDir(path.to_path_buf());

        // Before: doesn't exist
        path.assert(predicate::path::missing());

        action.apply()?;

        // Applied: exists
        path.assert(predicate::path::exists());

        action.undo()?;

        // Undone: doesn't exist
        path.assert(predicate::path::missing());

        Ok(())
    }

    #[test]
    fn test_remove_dir() -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.child("test");
        Action::MakeDir(path.to_path_buf()).apply()?;

        // Before: exists
        path.assert(predicate::path::exists());

        let rmdir_action = Action::RemoveDir(path.to_path_buf());

        rmdir_action.apply()?;

        // Applied: doesn't exist
        path.assert(predicate::path::missing());

        rmdir_action.undo()?;

        // Undone: exists
        path.assert(predicate::path::exists());

        Ok(())
    }

    #[test]
    fn test_move() -> Result<()> {
        let dir = TempDir::new()?;
        let source = dir.child("source");
        let target = dir.child("target");

        source.touch().unwrap();

        // Before: source exists, target doesn't
        source.assert(predicate::path::exists());
        target.assert(predicate::path::missing());

        let mv = Action::Move {
            source: source.to_path_buf(),
            target: target.to_path_buf(),
        };

        mv.apply()?;

        // Applied: source doesn't, target exists
        source.assert(predicate::path::missing());
        target.assert(predicate::path::exists());

        mv.undo()?;

        // Undone: source exists, target doesn't
        source.assert(predicate::path::exists());
        target.assert(predicate::path::missing());

        Ok(())
    }
}
