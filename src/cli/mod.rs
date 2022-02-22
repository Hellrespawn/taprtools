/// Parses command line arguments
pub(crate) mod args;
/// Handles commands
pub(crate) mod commands;
/// Contains filesystem code
pub(crate) mod fs;
/// Validate interpreted paths
pub(crate) mod validate;

pub(crate) use fs::Filesystem;

mod main;
pub use main::main;
