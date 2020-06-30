use gumdrop::Options;
use log::LevelFilter;
use log::{debug, log};
use std::env::args;
use std::fs;
use std::path;

use musictools::tfmt;

fn setup_logger(verbosity: usize) -> Result<(), fern::InitError> {
    let levels = [
        LevelFilter::Off,
        LevelFilter::Error,
        LevelFilter::Warn,
        LevelFilter::Info,
        LevelFilter::Debug,
        LevelFilter::Trace,
    ];

    // verbosity is usize, so can never be negative.
    if verbosity > levels.len() - 1 {
        panic!("Verbosity must be between 0 and {}", levels.len())
    }

    let level = levels[verbosity];

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        //.chain(std::io::stderr())
        .chain(fern::log_file("musictools.log")?)
        .apply()?;

    log!(
        log::max_level().to_level().unwrap_or(log::Level::Error),
        "Log started."
    );

    Ok(())
}

fn verbosity_from_args() -> usize {
    #[derive(Debug, Options)]
    struct VerbOptions {
        #[options(count, help = "increase a counting value")]
        verbosity: usize,
    }

    VerbOptions::parse_args_default_or_exit().verbosity
}

fn main() -> Result<(), String> {
    let verbosity = verbosity_from_args();

    setup_logger(verbosity).unwrap();

    debug!("Verbosity: {}", verbosity);

    println!("Running {:?}", args().next().unwrap());

    let mut path = path::PathBuf::from(file!());
    for _ in 1..=3 {
        path.pop();
    }
    path.push("tests");
    path.push("files");
    path.push("config");
    path.push("typical_input.tfmt");

    let test_string =
        fs::read_to_string(path).expect("typical_input.tfmt doesn't exist!");

    let mut lex = tfmt::lexer::Lexer::new(&test_string);

    let mut tokens: Vec<tfmt::token::Token> = Vec::new();

    loop {
        match lex.next_token() {
            Ok(Some(token)) => tokens.push(token),
            Ok(None) => {
                debug!("{:#?}", tokens);
                break;
            }
            Err(err) => {
                debug!("{:#?}", tokens);
                println!("Error: {}", err);
                break;
            }
        }
    }

    Ok(())
}
