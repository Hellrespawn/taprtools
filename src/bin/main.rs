use anyhow::Result;
use tfmttools::cli::tfmt;

fn main() -> Result<()> {
    // TODO? Type annotation is completely inconsequential here.
    tfmt::main::<&str>(None)
}
