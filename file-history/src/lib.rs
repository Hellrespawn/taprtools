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

mod actiongroup;
mod disk;

use actiongroup::ActionGroup;
use disk::DiskHandler;

pub use action::Action;
pub use error::HistoryError;
pub use history::History;

/// Wrapper for Result
pub type Result<T> = std::result::Result<T, HistoryError>;
