use anyhow::Result;
use maplit::hashmap;
use tfmttools::cli::rename::get_audiofiles;
use tfmttools::tfmt::interpreter::Interpreter;
use tfmttools::tfmt::lexer::{Lexer, LexerResult};
use tfmttools::tfmt::parser::Parser;
use tfmttools::tfmt::semantic::SymbolTable;

use std::path::PathBuf;
use std::str::FromStr;

mod common;

fn file_test(
    filename: &str,
    reference: &[&str],
    symbol_table: Option<SymbolTable>,
) -> Result<()> {

    let symbol_table = if let Some(symbol_table) = symbol_table {
        symbol_table
    } else {
        SymbolTable::new()
    };

    let input = common::get_script(filename)?;

    let tokens: Vec<LexerResult> = Lexer::from_str(&input)?.collect();

    let mut parser = Parser::from_iterator(tokens.into_iter());

    let program = parser.parse()?;

    let songs = get_audiofiles(&PathBuf::from("testdata/music"), 1)?;

    for song in songs {
        let output = Interpreter::new(
            song,
            &symbol_table
        )
        .interpret(&program)?;

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
        None,
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
        Some(hashmap! {
            "folder".to_string() => "destination".to_string()
        }),
    )
}
