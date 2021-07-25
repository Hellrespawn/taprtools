//! Tools to manage your music library using TagFormat.
//!
//! The TagFormat scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

/// CLI modules.
pub mod cli;

/// Crate errors.
pub mod error;

/// File handling modules.
pub mod file;

/// TagFormat-related modules.
pub mod tfmt;
