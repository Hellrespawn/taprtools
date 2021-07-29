use anyhow::Result;
use tfmttools::file::audiofile::get_audiofiles;
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
fn interpreter_simple_input_test() -> Result<()> {
    file_test(
        "simple_input.tfmt",
        &[
            "MASTER BOOT RECORD/Dune.mp3",
            "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
            "Amon Amarth/Under Siege.mp3",
            "Damjan Mravunac/Welcome To Heaven.ogg",
            "Nightwish/While Your Lips Are Still Red.mp3",
        ],
        &[],
    )
}

#[test]
fn interpreter_typical_input_test() -> Result<()> {
    common::init_logger();
    file_test(
        "typical_input.tfmt",
        &[
            "destination/MASTER BOOT RECORD/WAREZ/Dune.mp3",
            "destination/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
            "destination/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
            "destination/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
            "destination/Nightwish/While Your Lips Are Still Red.mp3",
        ],
        &["destination"],
    )
}
