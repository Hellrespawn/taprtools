use anyhow::Result;
use std::ffi::OsString;
use tfmttools::cli::tfmt;

fn intercept_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();

    let name = args
        .next()
        .expect("std::env::args_os() has no elements, not even a name!");
    [vec![name, OsString::from("--preview")], args.collect()].concat()
}

// TODO? Take out preview/dry-run entirely and just use the ptfmt target
// to pass a bool?
fn main() -> Result<()> {
    tfmt::main(&intercept_args(std::env::args_os()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ptfmt_test() {
        assert_eq!(
            intercept_args(
                "test.exe --h --as --some=args"
                    .split_whitespace()
                    .map(OsString::from)
            ),
            "test.exe --preview --h --as --some=args"
                .split_whitespace()
                .map(OsString::from)
                .collect::<Vec<OsString>>()
        );
    }
}
