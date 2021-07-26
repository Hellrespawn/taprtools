use crate::cli::{argparse, logging};
use crate::tfmt::ast;
use crate::tfmt::genastdot::GenAstDot;
use crate::tfmt::lexer::Lexer;
use crate::tfmt::parser::Parser;
use anyhow::Result;
use log::{debug, info};
use std::convert::TryInto;

/// Main tfmttools entrypoint.
pub fn main() -> Result<()> {
    let args = argparse::parse_args();

    let temp_dir = std::env::temp_dir();

    logging::setup_logger(args.verbosity.try_into()?, &temp_dir, "tfmttools")?;

    info!("{:#?}", args);

    let lex = lexer_test("typical_input.tfmt")?;
    let root = parser_test(lex)?;

    debug!("{:#?}", root);

    GenAstDot::visualize_ast(root, &temp_dir, "genastdot", true)?;

    Ok(())
}

fn lexer_test(filename: &str) -> Result<Lexer> {
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

    Ok(Lexer::new(&input)?)
}

fn parser_test(lex: Lexer) -> Result<ast::Program> {
    let mut p = Parser::from_iterator(lex);

    Ok(p.parse()?)
}
