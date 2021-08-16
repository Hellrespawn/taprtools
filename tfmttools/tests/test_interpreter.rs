use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use tfmttools::file::audio_file::get_audio_files;
use tfmttools::file::AudioFile;
use tfmttools::tfmt::ast::node::Program;
use tfmttools::tfmt::ast::Parser;
use tfmttools::tfmt::lexer::{Lexer, LexerResult};
use tfmttools::tfmt::visitors::{Interpreter, SemanticAnalyzer, SymbolTable};
use tfmttools::{helpers, RECURSION_DEPTH};

mod common;

fn seq_test(
    audio_files: &[Box<dyn AudioFile>],
    program: &Program,
    symbol_table: &SymbolTable,
    reference: &[String],
) -> Result<()> {
    let output: std::result::Result<
        Vec<String>,
        tfmttools::tfmt::error::InterpreterError,
    > = audio_files
        .iter()
        .map(|s| {
            Interpreter::new(&program, &symbol_table, s.as_ref()).interpret()
        })
        .collect();

    for string in output? {
        assert!(reference.contains(&string))
    }

    Ok(())
}

fn par_test(
    audio_files: &[Box<dyn AudioFile>],
    program: &Program,
    symbol_table: &SymbolTable,
    reference: &[String],
) -> Result<()> {
    let output: std::result::Result<
        Vec<String>,
        tfmttools::tfmt::error::InterpreterError,
    > = audio_files
        .par_iter()
        .map(|s| {
            Interpreter::new(program, symbol_table, s.as_ref()).interpret()
        })
        .collect();

    for string in output? {
        assert!(reference.contains(&string))
    }

    Ok(())
}

fn file_test(
    filename: &str,
    reference: &[&str],
    arguments: &[&str],
) -> Result<()> {
    let input = common::get_script(filename)?;

    let tokens: Vec<LexerResult> = Lexer::new(&input)?.collect();

    let mut parser = Parser::new(tokens.into_iter());

    let program = parser.parse()?;

    let symbol_table = SemanticAnalyzer::analyze(&program, arguments)?;

    let audio_files = get_audio_files(
        &PathBuf::from("testdata/music"),
        RECURSION_DEPTH,
        None,
    )?;

    let reference: Vec<String> = reference
        .iter()
        .map(helpers::normalize_separators)
        .collect();

    seq_test(&audio_files, &program, &symbol_table, &reference)?;

    par_test(&audio_files, &program, &symbol_table, &reference)?;

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
