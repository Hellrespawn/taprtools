use crate::cli::{argparse, logging};
use anyhow::Result;
use log::{debug, info};

pub fn main() -> Result<()> {
    let args = argparse::parse_args();

    let temp_dir = std::env::temp_dir();

    logging::setup_logger(args.verbosity, &temp_dir, "tfmttools")?;

    info!("{:#?}", args);

    test("simple_input.tfmt")
}

fn test(filename: &str) -> Result<()> {
    let mut path = std::path::PathBuf::from(file!());
    for _ in 1..=3 {
        path.pop();
    }
    path.push("tests");
    path.push("files");
    path.push("config");
    path.push(filename);

    let input = std::fs::read_to_string(path)
        .unwrap_or_else(|_| format!("{} doesn't exist!", filename));

    let mut lex = crate::tfmt::lexer::Lexer::new(&input);

    let mut tokens: Vec<crate::tfmt::token::Token> = Vec::new();
    while let Some(token) = lex.next_token()? {
        tokens.push(token);
    }

    debug!("tokens: {:#?}", tokens);

    Ok(())
}
