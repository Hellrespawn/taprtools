use crate::ast::node::{Node, Program};
use crate::visitors::{SemanticAnalyzer, SymbolTable};

use crate::ast::{Parser, Visitor};
use crate::error::ScriptError;

type Result<T> = std::result::Result<T, ScriptError>;

// FIXME Semantic Analyzer picks out name, parameters, stores them here

pub struct Script {
    pub input_text: String,
    name: String,
    description: String,
    parameters: Vec<String>,
    program: Program,
}

impl Script {
    pub fn new<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let input_text = input.as_ref().to_string();
        let mut parser = Parser::new(&input)?;
        let entry_point = parser.parse()?;

        let (name, description, parameters) =
            (String::new(), String::new(), Vec::new());

        // FIXME get arguments here.

        Ok(Script {
            input_text,
            name,
            description,
            parameters,
            program: entry_point,
        })
    }

    pub(crate) fn accept_visitor<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        self.program.accept(visitor)
    }
}
