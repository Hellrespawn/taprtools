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
pub(crate) use actiongroup::ActionGroup;

pub use action::Action;
pub use history::History;

pub use error::HistoryError;

/// Wrapper for Result
pub type Result<T> = std::result::Result<T, HistoryError>;

/// Titlecases `string`.
pub fn titlecase(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
