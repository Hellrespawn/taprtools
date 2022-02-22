#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]

//! Tools to manage your music library using `TagFormat`.
//!
//! The `TagFormat` scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

/// Controls the command line interface
pub mod cli;
mod file;
