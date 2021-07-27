use crate::cli::{argparse, logging};
use anyhow::Result;
use log::info;
use std::convert::TryInto;

/// Main tfmttools entrypoint.
pub fn main() -> Result<()> {
    let args = argparse::parse_args();

    logging::setup_logger(args.verbosity.try_into()?, "tfmttools")?;
    info!("Parsed arguments:\n{:#?}", &args);

    Ok(())
}
