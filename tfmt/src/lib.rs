#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
//! Constructs a string based on a script and audiofile tags

/// Abstract syntax tree
pub(crate) mod ast;
/// Functions
pub(crate) mod function;
/// `Lexer`
pub(crate) mod lexer;
/// `Token` and `TokenType`
pub(crate) mod token;

/// [`AudioFile`] trait
pub mod audio_file;
/// Crate errors.
pub mod error;
/// Script struct
pub mod script;
/// `Node` Visitors
pub mod visitors;

pub use audio_file::AudioFile;

use std::path::MAIN_SEPARATOR;

/// Normalizes newlines in `string`.
pub(crate) fn normalize_newlines<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalizes separators for the platform in `string`.
pub(crate) fn normalize_separators<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace(
        if MAIN_SEPARATOR == '/' { '\\' } else { '/' },
        &MAIN_SEPARATOR.to_string(),
    )
}
