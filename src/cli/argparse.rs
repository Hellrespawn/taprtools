use anyhow::{anyhow, bail, Result};
use clap::{load_yaml, App, ArgMatches};
use log::info;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Contains the collected and parsed command line arguments.
#[derive(Debug, PartialEq)]
pub struct Args {
    /// Verbosity
    pub verbosity: u64,
    /// Whether or not to actually rename files.
    pub dry_run: bool,
    /// Folder to read/write configuration to.
    pub config_folder: PathBuf,
    /// Arguments specific to chosen subcommand.
    pub subcommand: Subcommand,
}

impl Args {
    /// Accumulate arguments from submatches into main struct.
    #[allow(non_snake_case)]
    pub fn accumulate_ArgMatches(
        matches: &ArgMatches,
        verbosity: &mut u64,
        dry_run: &mut bool,
        config_folder: &mut Option<PathBuf>,
    ) {
        *verbosity += matches.occurrences_of("verbose");
        *dry_run |= matches.is_present("dry-run");
        if let Some(folder) = matches.value_of("config-folder") {
            *config_folder = Some(PathBuf::from(folder));
        }
    }
}

/// Contains the collected and parsed command line arguments specific to the
/// chosen subcommand.
#[derive(Debug, PartialEq)]
// TODO? Add a default subcommand?
pub enum Subcommand {
    /// Clear History.
    ClearHistory,
    /// List scripts.
    ListScripts,
    /// Redo `amount` actions.
    Redo(u64),
    /// Undo `amount` actions.
    Undo(u64),
    /// Inspect script `name`.
    Inspect {
        script_name: String,
        visualize: bool,
    },
    /// Rename files.
    Rename {
        script_name: String,
        arguments: Vec<String>,
        input_folder: PathBuf,
        output_folder: Option<PathBuf>,
        recursive: bool,
    },
}

impl Subcommand {
    fn from_subcommand(name: &str, submatches: &ArgMatches) -> Result<Self> {
        match name {
            "clear-history" => Ok(Subcommand::ClearHistory),
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
                visualize: submatches.is_present("visualize"),
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
                output_folder: submatches
                    .value_of("output-folder")
                    .map(PathBuf::from),
                recursive: submatches.is_present("recursive"),
            }),
            other => bail!("Unknown subcommand name: {}", other),
        }
    }
}

/// Parse arguments.
pub fn parse_args<S: AsRef<OsStr>>(args: &[S]) -> Result<Args> {
    let yaml = load_yaml!("tfmttools.yml");
    let matches = App::from_yaml(yaml).get_matches_from(args.iter());

    let (name, submatches) = matches.subcommand();

    // SubcommandRequired is enabled in tfmttools.yml
    let submatches = submatches.unwrap();

    let mut verbosity = 0;
    let mut dry_run = false;
    let mut config_folder = None;

    Args::accumulate_ArgMatches(
        &matches,
        &mut verbosity,
        &mut dry_run,
        &mut config_folder,
    );
    Args::accumulate_ArgMatches(
        submatches,
        &mut verbosity,
        &mut dry_run,
        &mut config_folder,
    );

    let args = Args {
        verbosity,
        dry_run,
        config_folder: get_config_folder(config_folder.as_ref(), dry_run)?,
        subcommand: Subcommand::from_subcommand(name, submatches)?,
    };

    Ok(args)
}

fn get_config_folder<P: AsRef<Path>>(
    config_folder: Option<&P>,
    dry_run: bool,
) -> Result<PathBuf> {
    let dir = if let Some(config_folder) = config_folder {
        Ok(PathBuf::from(config_folder.as_ref()))
    } else {
        dirs::home_dir()
            .map(|p| p.join(".tfmttools"))
            .or_else(|| dirs::config_dir().map(|p| p.join("tfmttools")))
            .ok_or_else(|| {
                anyhow!("Unable to find valid configuration directory!")
            })
    }?;

    if !dir.exists() {
        if !dry_run {
            std::fs::create_dir_all(&dir)?;

            let s = format!(
                "Creating configuration directory at \"{}\"",
                dir.to_string_lossy()
            );

            println!("{}", s);
            info!("{}", s);
        }
    } else if !dir.is_dir() {
        bail!("{} is not a folder!", dir.to_string_lossy())
    }

    Ok(dir)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn argparse_test() -> Result<()> {
        // Don't forget program name!
        let cli_args =
            "argparse.exe -vv --dry-run rename -vv Sync -- these are arguments";
        let test_args = Args {
            verbosity: 4,
            dry_run: true,
            config_folder: PathBuf::new(),
            subcommand: Subcommand::Rename {
                script_name: "Sync".to_string(),
                arguments: vec![
                    "these".to_string(),
                    "are".to_string(),
                    "arguments".to_string(),
                ],
                input_folder: std::env::current_dir()?,
                output_folder: None,
                recursive: false,
            },
        };

        let mut parsed_args =
            parse_args(&cli_args.split_whitespace().collect::<Vec<&str>>())?;
        parsed_args.config_folder = PathBuf::new();

        assert_eq!(parsed_args, test_args);

        Ok(())
    }
}
