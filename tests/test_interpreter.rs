use anyhow::Result;
use tfmttools::tfmt::interpreter::Interpreter;
use tfmttools::tfmt::lexer::Lexer;
use tfmttools::tfmt::parser::Parser;

mod common;

fn file_test(filename: &str, reference: &[&str]) -> Result<()> {
    let input = common::get_script(filename)?;

    let program = Parser::<Lexer>::from_string(&input)?.parse()?;

    let songs = common::get_songs()?;

    for song in songs {
        let output = Interpreter::new(song).interpret(&program)?;

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
    )
}

#[test]
fn test_typical_input() -> Result<()> {
    file_test(
        "typical_input.tfmt",
        &[
            r"folder/MASTER BOOT RECORD/WAREZ/Dune",
            r"folder/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1",
            r"folder/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege",
            r"folder/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven",
            r"folder/Nightwish/While Your Lips Are Still Red",
        ],
    )
}
