#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]

//! Tools to manage your music library using `TagFormat`.
//!
//! The `TagFormat` scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

// TODO? Write proper config/settings module?

mod cli;
mod helpers;
mod tags;

const HISTORY_FILENAME: &str = "tfmttools.hist";

/// Number of [`AudioFile`]s to preview.
pub const PREVIEW_AMOUNT: usize = 8;

/// Number of folders to recurse from --input-dir
pub const RECURSION_DEPTH: u64 = 4;

pub const PREVIEW_PREFIX: &str = "[P] ";
