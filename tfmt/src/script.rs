use crate::ast::node::Program;
use crate::visitors::{SymbolTable, SemanticAnalyzer};

use crate::error::ScriptError;
use crate::ast::Parser;

type Result<T> = std::result::Result<T, ScriptError>;

pub(crate) struct Script {
    input_text: String,
    program: Program,
    // symbol_table: SymbolTable,
}

impl Script {
    pub(crate) fn new<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let input_text = input.as_ref().to_string();
        let mut parser = Parser::new(&input)?;
        let program = parser.parse()?;

        // FIXME get arguments here.

        Ok(Script {
            input_text,
            program
        })
    }
}
