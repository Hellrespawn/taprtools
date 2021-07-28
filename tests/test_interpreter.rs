use anyhow::Result;
use tfmttools::cli::rename::get_audiofiles;
use tfmttools::tfmt::interpreter::Interpreter;
use tfmttools::tfmt::lexer::{Lexer, LexerResult};
use tfmttools::tfmt::parser::Parser;

use std::path::PathBuf;
use std::str::FromStr;

mod common;

fn file_test(
    filename: &str,
    reference: &[&str],
    arguments: &[&str],
) -> Result<()> {
    let input = common::get_script(filename)?;

    let tokens: Vec<LexerResult> = Lexer::from_str(&input)?.collect();

    let mut parser = Parser::from_iterator(tokens.into_iter());

    let program = parser.parse()?;

    let songs = get_audiofiles(&PathBuf::from("testdata/music"), 1)?;

    let mut intp = Interpreter::new(&program, arguments, &songs)?;

    for output in intp.interpret()? {
        assert!(reference.contains(&output.as_str()))
    }

    Ok(())
}

#[test]
fn test_simple_input() -> Result<()> {
    file_test(
        "simple_input.tfmt",
        &[
            r"MASTER BOOT RECORD/Dune",
            r"MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1",
            r"Amon Amarth/Under Siege",
            r"Damjan Mravunac/Welcome To Heaven",
            r"Nightwish/While Your Lips Are Still Red",
        ],
        &[],
    )
}

#[test]
fn test_typical_input() -> Result<()> {
    common::init_logger();
    file_test(
        "typical_input.tfmt",
        &[
            r"destination/MASTER BOOT RECORD/WAREZ/Dune",
            r"destination/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1",
            r"destination/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege",
            r"destination/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven",
            r"destination/Nightwish/While Your Lips Are Still Red",
        ],
        &["destination"],
    )
}
