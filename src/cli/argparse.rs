use std::u64;

use clap::{load_yaml, App, ArgMatches};
use std::ffi::OsString;

/// Contains the collected and parsed command line arguments.
#[derive(Debug, Default, PartialEq)]
pub struct Args {
    /// Verbosity
    pub verbosity: u64,
    /// Whether or not to actually rename files.
    pub dry_run: bool,
    /// Arguments specific to chosen subcommand.
    pub sub_args: Option<SubArgs>,
}

impl Args {
    /// Accumulate arguments from submatches into main struct.
    #[allow(non_snake_case)]
    pub fn accumulate_ArgMatches(&mut self, matches: &ArgMatches) {
        self.verbosity += matches.occurrences_of("verbose");
        self.dry_run |= matches.is_present("dry-run");
    }
}

/// Contains the collected and parsed command line arguments specific to the
/// chosen subcommand.
#[derive(Debug, PartialEq)]
pub enum SubArgs {
    /// Undo `amount` actions.
    Undo { amount: u64 },
    /// Redo `amount` actions.
    Redo { amount: u64 },
    /// Inspect script `name`.
    Inspect { name: String },
    /// Rename files.
    Rename {
        name: String,
        arguments: Option<Vec<String>>,
        recursive: bool,
        allow_case_difference: bool,
    },
}

impl SubArgs {
    fn from_subcommand(name: &str, submatches: &ArgMatches) -> Option<Self> {
        match name {
            "undo" => Some(SubArgs::Undo {
                amount: submatches
                    .value_of("amount")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Invalid amount!"),
            }),
            "redo" => Some(SubArgs::Redo {
                amount: submatches
                    .value_of("amount")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Invalid amount!"),
            }),
            "inspect" => Some(SubArgs::Inspect {
                name: submatches
                    .value_of("name")
                    .expect("Name wasn't specified!")
                    .to_string(),
            }),
            "rename" => Some(SubArgs::Rename {
                name: submatches
                    .value_of("name")
                    .expect("Name wasn't specified!")
                    .to_string(),
                // Option::map maps Option<T> to Option<U>
                // Iterator::map items in iterator
                arguments: submatches
                    .values_of("arguments")
                    .map(|i| i.map(String::from).collect()),
                allow_case_difference: submatches
                    .is_present("allow-case-difference"),
                recursive: submatches.is_present("recursive"),
            }),
            _ => None,
        }
    }
}

/// Wrapper function for [`_parse_args`] using command line arguments..
pub fn parse_args() -> Args {
    _parse_args(std::env::args_os())
}

/// Parse arguments.
fn _parse_args<I, T>(iterator: I) -> Args
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let yaml = load_yaml!("tfmttools.yml");
    let matches = App::from_yaml(yaml).get_matches_from(iterator);

    let mut args: Args = Default::default();
    args.accumulate_ArgMatches(&matches);

    if let Some(name) = matches.subcommand_name() {
        let submatches = matches.subcommand_matches(name).unwrap();

        args.accumulate_ArgMatches(&submatches);
        args.sub_args = SubArgs::from_subcommand(name, submatches);
    }

    args
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_argparser() {
        // Don't forget program name!
        let cli_args =
            "argparse.exe -vv rename -vv Sync -- these are arguments";
        let test_args = Args {
            verbosity: 4,
            dry_run: false,
            sub_args: Some(SubArgs::Rename {
                name: "Sync".to_string(),
                arguments: Some(vec![
                    "these".to_string(),
                    "are".to_string(),
                    "arguments".to_string(),
                ]),
                recursive: false,
                allow_case_difference: false,
            }),
        };

        assert_eq!(_parse_args(cli_args.split_whitespace()), test_args)
    }
}
