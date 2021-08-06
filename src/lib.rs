//#![warn(missing_docs)]
//! Tools to manage your music library using TagFormat.
//!
//! The TagFormat scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

// TODO? Write proper config/settings module?
pub const PREVIEW_AMOUNT: usize = 8;
pub const RECURSION_DEPTH: u64 = 4;

/// CLI modules.
pub mod cli;
/// Crate errors.
pub mod error;
/// File handling modules.
pub mod file;
/// Helpers
pub mod helpers;
/// TagFormat-related modules.
pub mod tfmt;
