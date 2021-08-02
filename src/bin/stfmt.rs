use anyhow::Result;
use std::ffi::OsString;
use tfmttools::cli::tfmt;

fn intercept_args() -> Vec<OsString> {
    let mut args = std::env::args_os();

    let name = args
        .next()
        .expect("std::env::args_os() has no elements, not even a name!");
    [vec![name, OsString::from("--dry-run")], args.collect()].concat()

}

fn main() -> Result<()> {
    tfmt::main(&intercept_args())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stfmt_test() {
        // FIXME Write this test
    }
}
