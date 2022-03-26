use crate::cli::Config;
use anyhow::Result;
use clap::Parser;
use file_history::History;
use std::path::PathBuf;

#[derive(Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sets a custom config file
    #[clap(parse(from_os_str))]
    histfile: Option<PathBuf>,
}

/// Entry point for histviewer
pub fn histviewer() -> Result<()> {
    let args = Args::parse();

    let history_path = if let Some(path) = args.histfile {
        path
    } else {
        Config::new(&Config::default_path()?)?.get_history_path()
    };

    let history = History::load(&history_path)?;

    println!("{}", history);

    Ok(())
}
