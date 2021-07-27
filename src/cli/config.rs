use crate::tfmt::ast::Program;
use crate::tfmt::parser::Parser;
use anyhow::Result;
use std::convert::TryFrom;
use std::path::PathBuf;

pub fn read_script(name: &str) -> Result<(PathBuf, Program)> {
    // FIXME actually implement read_script
    let path = PathBuf::from("testdata/script/typical_input.tfmt");

    let program = Parser::try_from(&path)?.parse()?;

    Ok((path, program))
}
