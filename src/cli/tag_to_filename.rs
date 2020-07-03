use log::{debug, trace};
use std::env::args;
use std::fs;
use std::path;

use crate::{logger, tfmt};

pub fn main() -> Result<(), String> {
    let verbosity = logger::verbosity_from_args();

    if let Err(err) = logger::setup_logger(
        verbosity,
        logger::path_relative_to_source_file(),
        "musictools",
    ) {
        panic!("Unable to initialize logger: {}", err)
    };

    debug!("Verbosity: {}", verbosity);

    println!("Running {:?}", args().next().unwrap());

    let filename = "simple_input.tfmt";
    //let filename = "typical_input.tfmt";

    let mut path = path::PathBuf::from(file!());
    for _ in 1..=3 {
        path.pop();
    }
    path.push("tests");
    path.push("files");
    path.push("config");
    path.push(filename);

    let test_string = fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("{} doesn't exist!", filename));

    let mut lex = tfmt::lexer::Lexer::new(&test_string);

    let mut tokens: Vec<tfmt::token::Token> = Vec::new();

    loop {
        match lex.next_token() {
            Ok(Some(token)) => {
                trace!("{:?}", token);
                tokens.push(token);
            }
            Ok(None) => {
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    lex.reset();

    let mut parser = tfmt::parser::Parser::from_lexer(lex);

    let root = parser.parse()?;

    let mut path = path::PathBuf::from(file!());
    for _ in 1..=3 {
        path.pop();
    }
    path.push("log");

    if tfmt::genastdot::visualize_ast(root, &path, "musictools", false).is_err() {
        panic!("viz failed!");
    }

    Ok(())
}
