//#![warn(missing_docs)]
//! Tools to manage your music library using TagFormat.
//!
//! The TagFormat scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

// TODO? Write proper config/settings module?

const HISTORY_FILENAME: &str = "tfmttools.hist";

/// Number of [AudioFile]s to preview.
pub const PREVIEW_AMOUNT: usize = 8;

/// Number of folders to recurse from --input-dir
pub const RECURSION_DEPTH: u64 = 4;

/// TODO Mock this for tests.
/// Minimum files before switching to parallel
// pub const MIN_PARALLEL: usize = 128;

/// CLI modules.
pub mod cli;
/// File handling modules.
pub mod file;
/// GUI
pub mod gui;
/// Helpers
pub mod helpers;
/// TagFormat-related modules.
pub mod tfmt;
