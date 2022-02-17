use crate::ast::node::{Node, Program};
use crate::visitors::{SemanticAnalyzer, ScriptParameter};

use crate::ast::{Parser, Visitor};
use crate::error::ScriptError;

type Result<T> = std::result::Result<T, ScriptError>;

// FIXME Semantic Analyzer picks out name, parameters, stores them here

pub struct Script {
    pub input_text: String,
    name: String,
    description: String,
    parameters: Vec<ScriptParameter>,
    program: Program,
}

impl Script {
    pub fn new<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let input_text = input.as_ref().to_string();
        let mut parser = Parser::new(&input)?;
        let program = parser.parse()?;

        let (name, description, parameters) = SemanticAnalyzer::analyze(&program)?;

        Ok(Script {
            input_text,
            name,
            description,
            parameters,
            program,
        })
    }

    pub(crate) fn accept_visitor<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        self.program.accept(visitor)
    }

    // pub(crate) fn check_arguments() {


    //     // Check that we have the right amount of arguments
    //     if arguments.len() > analyzer.symbols.len() {
    //         return Err(SemanticError::TooManyArguments {
    //             found: arguments.len(),
    //             expected: analyzer.symbols.len(),
    //             name: analyzer.name,
    //         });
    //     }




    //     for (key, val) in &output {
    //         if val.is_none() {
    //             return Err(SemanticError::ArgumentRequired(
    //                 // clippy::inefficient_to_string
    //                 (*key).to_string(),
    //                 analyzer.name,
    //             ));
    //         }
    //     }
    // }
}
