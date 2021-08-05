use crate::helpers::pp;
use anyhow::{anyhow, bail, Result};
use clap::{load_yaml, App, ArgMatches};
use log::info;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Contains the collected and parsed command line arguments.
#[derive(Debug, PartialEq)]
pub struct Args {
    /// Verbosity
    pub verbosity: u64,
    /// Whether or not to actually rename files.
    pub preview: bool,
    /// Folder to read/write configuration to.
    pub config_folder: PathBuf,
    /// Arguments specific to chosen subcommand.
    pub subcommand: Subcommand,
}

impl Args {
    /// Accumulate arguments from submatches into main struct.
    pub fn accumulate_arg_matches(
        matches: &ArgMatches,
        verbosity: &mut u64,
        preview: &mut bool,
        config_folder: &mut Option<PathBuf>,
    ) {
        *verbosity += matches.occurrences_of("verbose");
        *preview |= matches.is_present("preview");
        //TODO? Error on double config-folder?
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
        render_ast: bool,
    },
    /// Rename files.
    Rename {
        script_name: String,
        arguments: Vec<String>,
        input_folder: PathBuf,
        output_folder: PathBuf,
        recursive: bool,
    },
}

impl Subcommand {
    fn from_str(name: &str, submatches: &ArgMatches) -> Result<Self> {
        match name {
            "clear-history" => Ok(Subcommand::ClearHistory),
            "list-scripts" => Ok(Subcommand::ListScripts),
            "redo" => Ok(Subcommand::Redo(
                submatches.value_of("amount").unwrap().parse::<u64>()?,
            )),
            "undo" => Ok(Subcommand::Undo(
                submatches.value_of("amount").unwrap().parse::<u64>()?,
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
                output_folder: submatches
                    .value_of("output-folder")
                    .map(PathBuf::from)
                    .unwrap_or_else(PathBuf::new),
                recursive: submatches.is_present("recursive"),
            }),
            other => bail!("Unknown subcommand name: {}", other),
        }
    }
}

/// Parse arguments.
pub fn parse_args<I, O>(args: I) -> Result<Args>
where
    I: IntoIterator<Item = O>,
    O: Into<OsString> + Clone,
{
    let yaml = load_yaml!("tfmt.yml");
    let matches = App::from_yaml(yaml)
        .name(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .get_matches_from(args);

    let (name, submatches) = matches.subcommand();

    // SubcommandRequired is enabled in tfmttools.yml
    debug_assert!(submatches.is_some());
    let submatches = submatches.unwrap();

    let mut verbosity = 0;
    let mut preview = false;
    let mut config_folder = None;

    Args::accumulate_arg_matches(
        &matches,
        &mut verbosity,
        &mut preview,
        &mut config_folder,
    );
    Args::accumulate_arg_matches(
        submatches,
        &mut verbosity,
        &mut preview,
        &mut config_folder,
    );

    let args = Args {
        verbosity,
        preview,
        config_folder: get_config_folder(config_folder.as_ref(), preview)?,
        subcommand: Subcommand::from_str(name, submatches)?,
    };

    Ok(args)
}

// TODO? Move creation of folder away from determination of folder?
fn get_config_folder<P: AsRef<Path>>(
    config_folder: Option<&P>,
    preview: bool,
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
        if !preview {
            std::fs::create_dir_all(&dir)?;

            let s = format!(
                r#"{} Creating configuration directory at "{}""#,
                pp(preview),
                dir.to_string_lossy()
            );

            println!("{}", s);
            info!("{}", s);
        }
    } else if !dir.is_dir() {
        bail!("{} exists but is not a folder!", dir.to_string_lossy())
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
            "argparse.exe -vv --preview rename -vv Sync -- these are arguments";
        let test_args = Args {
            verbosity: 4,
            preview: true,
            config_folder: PathBuf::new(),
            subcommand: Subcommand::Rename {
                script_name: "Sync".to_string(),
                arguments: vec![
                    "these".to_string(),
                    "are".to_string(),
                    "arguments".to_string(),
                ],
                input_folder: std::env::current_dir()?,
                output_folder: PathBuf::new(),
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
