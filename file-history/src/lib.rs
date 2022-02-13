//#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
pub mod action;
pub mod error;
pub mod history;

pub use action::Action;
pub use error::HistoryError;

pub type Result<T> = std::result::Result<T, HistoryError>;

/// Titlecases `string`.
pub fn titlecase(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
