/// Parses command line arguments
pub(crate) mod args;
/// Handles commands
pub(crate) mod commands;
/// Contains configuration code
pub(crate) mod config;
/// Validate interpreted paths
pub(crate) mod validate;

use anyhow::Result;

/// Main entrypoint for tfmttools
pub fn main(preview_override: bool) -> Result<()> {
    let args = crate::cli::args::parse_args(preview_override);
    dbg!(args);
    println!("tfmt CLI");
    Ok(())
}
