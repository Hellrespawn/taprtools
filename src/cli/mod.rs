/// Parses command line arguments
pub(crate) mod args;
/// Handles commands
pub(crate) mod commands;
/// Contains filesystem code
pub(crate) mod config;
/// Validate interpreted paths
pub(crate) mod validate;

pub(crate) use config::Config;

mod main;
pub use main::main;
