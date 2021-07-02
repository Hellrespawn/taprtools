use std::u64;

use anyhow::Result;
use clap::{load_yaml, App, ArgMatches};

#[derive(Debug)]
pub enum SubArgs {
    Undo { amount: u64 },
    Redo { amount: u64 },
    Inspect { name: String },
    Rename {},
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
    let yaml = load_yaml!("tag_to_filename.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut args: Args = Default::default();
    args.accumulate_ArgMatches(&matches);

    if let Some(name) = matches.subcommand_name() {
        let submatches = matches.subcommand_matches(name).unwrap();
        args.accumulate_ArgMatches(&submatches);

        args.sub_args = match name {
            "undo" => Some(SubArgs::Undo { amount: 1 }),
            "redo" => Some(SubArgs::Redo { amount: 1 }),
            "inspect" => Some(SubArgs::Inspect {
                name: "test".to_string(),
            }),
            _ => None,
        };
    }

    println!("{:#?}", args);

    Ok(())
}
