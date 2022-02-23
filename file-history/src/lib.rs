#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
//! This crate tracks moving of files and creation and deletion of folders in a reversible

/// Contains [`Action`]
pub mod action;
/// Contains [`History`]
pub mod history;

mod actiongroup;
mod disk;

use actiongroup::ActionGroup;
use disk::DiskHandler;
use thiserror::Error;

pub use action::Action;
pub use history::History;

/// Wrapper for Result
pub type Result<T> = std::result::Result<T, HistoryError>;

#[derive(Error, Debug)]
/// Error relating to file-history
pub enum HistoryError {
    /// Represents std::io::Error
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Represents bincode::Error
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Represents a generic error
    #[error("{0}")]
    Generic(String),
}
