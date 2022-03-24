#![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::doc_markdown)]
//! Constructs a string based on a script and audiofile tags

mod ast;
mod error;
mod function;
mod lexer;
mod script;
mod tags;
mod token;
mod visitor;

pub use script::Script;
pub use tags::Tags;
pub use visitor::Interpreter;

use std::path::MAIN_SEPARATOR;

/// Forbidden graphemes that are part of TFMT.
pub(crate) const FORBIDDEN_GRAPHEMES: [&str; 8] =
    ["<", ">", ":", "\"", "|", "?", "*", "~"];

/// Directory separators.
pub(crate) static DIRECTORY_SEPARATORS: [&str; 2] = ["/", "\\"];

/// Normalizes newlines in `string`.
pub(crate) fn normalize_eol(string: &str) -> String {
    string.replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalizes separators for the platform in `string`.
pub(crate) fn normalize_separators(string: &str) -> String {
    string.replace(
        if MAIN_SEPARATOR == '/' { '\\' } else { '/' },
        &MAIN_SEPARATOR.to_string(),
    )
}
