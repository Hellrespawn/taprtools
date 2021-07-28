use anyhow::{bail, Result};
use clap::{load_yaml, App, ArgMatches};
use std::ffi::OsString;
use std::path::PathBuf;

/// Contains the collected and parsed command line arguments.
#[derive(Debug, PartialEq)]
pub struct Args {
    /// Verbosity
    pub verbosity: u64,
    /// Whether or not to actually rename files.
    pub dry_run: bool,
    /// Arguments specific to chosen subcommand.
    pub subcommand: Subcommand,
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
// TODO? Add a default subcommand?
pub enum Subcommand {
    /// List scripts.
    ListScripts,
    /// Redo `amount` actions.
    Redo(u64),
    /// Undo `amount` actions.
    Undo(u64),
    /// Inspect script `name`.
    Inspect {
        script_name: String,
        render_ast: bool,
    },
    /// Rename files.
    Rename {
        script_name: String,
        arguments: Vec<String>,
        input_folder: PathBuf,
        recursive: bool,
    },
}

impl Subcommand {
    fn from_subcommand(name: &str, submatches: &ArgMatches) -> Result<Self> {
        match name {
            "list-scripts" => Ok(Subcommand::ListScripts),
            "redo" => Ok(Subcommand::Redo(
                submatches
                    .value_of("amount")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Invalid amount!"),
            )),
            "undo" => Ok(Subcommand::Undo(
                submatches
                    .value_of("amount")
                    .unwrap()
                    .parse::<u64>()
                    .expect("Invalid amount!"),
            )),
            "inspect" => Ok(Subcommand::Inspect {
                script_name: submatches
                    .value_of("name")
                    .expect("Name wasn't specified!")
                    .to_string(),
                render_ast: submatches.is_present("render-ast"),
            }),
            "rename" => Ok(Subcommand::Rename {
                script_name: submatches
                    .value_of("script-name")
                    .expect("Name wasn't specified!")
                    .to_string(),
                // Option::map maps Option<T> to Option<U>
                // Iterator::map items in iterator
                arguments: submatches
                    .values_of("arguments")
                    .unwrap_or_default()
                    .map(String::from)
                    .collect(),
                input_folder: submatches
                    .value_of("input-folder")
                    .map(PathBuf::from)
                    .unwrap_or(std::env::current_dir()?),
                recursive: submatches.is_present("recursive"),
            }),
            other => bail!("Unknown subcommand name: {}", other),
        }
    }
}

/// Wrapper function for [`_parse_args`] using command line arguments..
pub fn parse_args() -> Result<Args> {
    _parse_args(std::env::args_os())
}

/// Parse arguments.
fn _parse_args<I, T>(iterator: I) -> Result<Args>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let yaml = load_yaml!("tfmttools.yml");
    let matches = App::from_yaml(yaml).get_matches_from(iterator);

    let (name, submatches) = matches.subcommand();
    let submatches = submatches.unwrap();

    let mut args = Args {
        verbosity: 0,
        dry_run: false,
        // SubcommandRequired is enabled in tfmttools.yml
        subcommand: Subcommand::from_subcommand(name, submatches)?,
    };

    args.accumulate_ArgMatches(&matches);
    args.accumulate_ArgMatches(submatches);

    Ok(args)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_argparser() -> Result<()> {
        // Don't forget program name!
        let cli_args =
            "argparse.exe -vv rename -vv Sync -- these are arguments";
        let test_args = Args {
            verbosity: 4,
            dry_run: false,
            subcommand: Subcommand::Rename {
                script_name: "Sync".to_string(),
                arguments: vec![
                    "these".to_string(),
                    "are".to_string(),
                    "arguments".to_string(),
                ],
                input_folder: std::env::current_dir()?,
                recursive: false,
            },
        };

        assert_eq!(_parse_args(cli_args.split_whitespace())?, test_args);

        Ok(())
    }
}
