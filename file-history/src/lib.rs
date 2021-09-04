pub mod action;
pub mod error;
pub mod history;

pub use action::Action;
pub use error::HistoryError;
pub use history::{ActionGroup, History, Stack};

pub type Result<T> = std::result::Result<T, error::HistoryError>;

/// Titlecases `string`.
pub fn titlecase(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
