use anyhow::Result;
use std::ffi::OsString;
use tfmttools::cli::tfmt;

fn main() -> Result<()> {
    let mut args = std::env::args_os();

    let name = args
        .next()
        .expect("std::env::args_os() has no elements, not even a name!");
    let new =
        [vec![name, OsString::from("--dry-run")], args.collect()].concat();

    tfmt::main(&new)
}
