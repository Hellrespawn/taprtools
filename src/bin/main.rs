use anyhow::Result;
use tfmttools::cli::tfmttools;

fn main() -> Result<()> {
    // TODO? Type annotation is completely inconsequential here.
    tfmttools::main::<&str>(None)
}
