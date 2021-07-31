use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;
use tfmttools::cli::tfmttools::get_audio_files;
use tfmttools::tfmt::interpreter::Interpreter;
use tfmttools::tfmt::lexer::{Lexer, LexerResult};
use tfmttools::tfmt::parser::Parser;
use tfmttools::tfmt::semantic::SemanticAnalyzer;

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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

    let symbol_table = SemanticAnalyzer::analyze(&program, arguments)?;

    let mut audio_files = Vec::new();

    get_audio_files(
        &mut audio_files,
        &PathBuf::from("testdata/music"),
        1,
        None,
    )?;

    #[cfg(feature = "rayon")]
    let iter = audio_files.par_iter();

    #[cfg(not(feature = "rayon"))]
    let iter = audio_files.iter();

    let output: std::result::Result<
        Vec<String>,
        tfmttools::error::InterpreterError,
    > = iter
        .map(|s| {
            Interpreter::new(&program, &symbol_table, s.as_ref()).interpret()
        })
        .collect();

    for string in output? {
        assert!(reference.contains(&string.as_str()))
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
