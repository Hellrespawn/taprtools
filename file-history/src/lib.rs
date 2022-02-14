#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
//! This crate tracks moving of files and creation and deletion of folders in a reversible

/// Contains [Action]
pub mod action;
/// Contains [HistoryError]
pub mod error;
/// Contains [History]
pub mod history;

pub(crate) mod actiongroup;
pub(crate) mod database;
pub(crate) use actiongroup::ActionGroup;
pub(crate) use database::Database;

pub use action::Action;
pub use history::History;

pub use error::HistoryError;

/// FIXME Test function
pub fn test_db() -> Result<()> {
    database::Database::connect("d:\\test.sqlite")?;
    Ok(())
}

/// Wrapper for Result
pub type Result<T> = std::result::Result<T, HistoryError>;
