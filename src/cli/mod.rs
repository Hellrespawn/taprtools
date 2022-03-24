/// Parses command line arguments
pub(crate) mod args;
/// Handles commands
pub(crate) mod commands;
/// Contains filesystem code
pub(crate) mod config;
/// Contains UI code
pub(crate) mod ui;

pub(crate) use config::Config;

mod histviewer;
mod main;
pub use args::Args;
pub use histviewer::histviewer;
pub use main::main;
