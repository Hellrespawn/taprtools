use crate::ast::node::{Node, Program};
use crate::visitor::dot::DotGenerator;
use crate::visitor::semantic::{Analysis, SemanticAnalyzer};
use crate::visitor::Visitor;

use crate::ast::Parser;
use crate::error::ScriptError;

type Result<T> = std::result::Result<T, ScriptError>;

#[derive(Debug)]
pub struct ScriptParameter {
    name: String,
    default: Option<String>,
    count: u64,
}

impl ScriptParameter {
    pub(crate) fn new(name: String, default: Option<String>) -> Self {
        ScriptParameter {
            name,
            default,
            count: 0,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }

    pub(crate) fn count(&mut self) -> &mut u64 {
        &mut self.count
    }
}

/// Reads a script, parses an AST and gets the name, description and parameters.
pub struct Script {
    input_text: String,
    name: String,
    description: Option<String>,
    parameters: Vec<ScriptParameter>,
    program: Program,
}

impl Script {
    /// Create a new Script instance.
    pub fn new<S>(input: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let input_text = input.as_ref().to_string();
        let mut parser = Parser::new(&input)?;
        let program = parser.parse()?;

        let Analysis {
            name,
            description,
            parameters,
        } = SemanticAnalyzer::analyze(&program)?;

        Ok(Script {
            input_text,
            name,
            description,
            parameters,
            program,
        })
    }

    /// Returns the original input text.
    pub fn input_text(&self) -> &str {
        &self.input_text
    }

    /// Returns the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the parameters
    pub fn parameters(&self) -> &[ScriptParameter] {
        &self.parameters
    }

    /// Accepts a visitor
    pub(crate) fn accept_visitor<T>(&self, visitor: &mut dyn Visitor<T>) -> T {
        self.program.accept(visitor)
    }

    /// Generate GraphViz .dot file of the Abstract Syntax Tree
    pub fn create_ast_dot(&self) -> String {
        DotGenerator::create_ast_dot(&self.program)
    }
}
