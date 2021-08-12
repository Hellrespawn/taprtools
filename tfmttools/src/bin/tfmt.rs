use anyhow::Result;
use std::ffi::OsString;
use tfmttools::cli::tfmt;

fn main() -> Result<()> {
    // TODO? Type annotation is completely inconsequential here.
    tfmt::main(&std::env::args_os().collect::<Vec<OsString>>())
}
