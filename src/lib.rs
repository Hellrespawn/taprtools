// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
//#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Tools to manage your music library using `TagFormat`.
//!
//! The `TagFormat` scripting language lets you write scripts to dynamically
//! rename your music files based on their tags.

/// Controls the command line interface
pub mod cli;
mod file;
mod script;
mod tags;
mod tapr;

// TODO Update `indicatif` to 0.17
// TODO Use `camino` to read files
// TODO Check if leftovers are images and offer to delete.

// TODO Show location in script on error

// TODO? Update tag with leading/trailing whitespace?
// TODO? Separate Move ActionType into CopyFile and RemoveFile?
// TODO? Add more obscure tags?
// TODO? Add separate strict mode, which errors on forbidden characters/directory separators instead of replacing.
