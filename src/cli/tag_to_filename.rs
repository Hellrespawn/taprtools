use dirs::home_dir;
use log::{debug, trace};
use std::env::args;
use std::fs;
use std::path;

use crate::{logger, tfmt};

pub fn main() -> Result<(), String> {
    let verbosity = logger::verbosity_from_args();

    let mut project_path = home_dir()
        .expect("Unable to find home folder!")
        .canonicalize()
        .unwrap();
    project_path.push("projects");
    project_path.push("rust");
    project_path.push("musictools_rust");

    let mut log_folder = path::PathBuf::from(&project_path);
    log_folder.push("log");

    if let Err(err) = logger::setup_logger(verbosity, log_folder, "musictools")
    {
        panic!("Unable to initialize logger: {}", err)
    };

    debug!("Verbosity: {}", verbosity);

    println!("Running {:?}", args().next().unwrap());

    //let filename = "simple_input.tfmt";
    let filename = "typical_input.tfmt";

    let mut input_file = path::PathBuf::from(&project_path);
    input_file.push("tests");
    input_file.push("files");
    input_file.push("config");
    input_file.push(filename);

    let test_string = fs::read_to_string(&input_file)
        .unwrap_or_else(|_| panic!("{:?} doesn't exist!", &input_file));

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

    if let Err(error) = tfmt::genastdot::visualize_ast(root, &path, "musictools", false) {
        println!("{}", error)
    }

    Ok(())
}
