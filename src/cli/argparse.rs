use std::u64;

use anyhow::Result;
use clap::{load_yaml, App, ArgMatches};

#[derive(Debug)]
pub enum SubArgs {
    Undo {
        amount: u64,
    },
    Redo {
        amount: u64,
    },
    Inspect {
        name: String,
    },
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

#[derive(Debug, Default)]
pub struct Args {
    verbose: u64,
    dry_run: bool,
    sub_args: Option<SubArgs>,
}

impl Args {
    #[allow(non_snake_case)]
    pub fn accumulate_ArgMatches(&mut self, matches: &ArgMatches) {
        self.verbose += matches.occurrences_of("verbose");
        self.dry_run |= matches.is_present("dry-run");
    }
}

pub fn parse_args() -> Result<()> {
    let yaml = load_yaml!("tfmttools.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut args: Args = Default::default();
    args.accumulate_ArgMatches(&matches);

    if let Some(name) = matches.subcommand_name() {
        let submatches = matches.subcommand_matches(name).unwrap();

        args.accumulate_ArgMatches(&submatches);
        args.sub_args = SubArgs::from_subcommand(name, submatches);
    }

    println!("{:#?}", args);

    Ok(())
}
