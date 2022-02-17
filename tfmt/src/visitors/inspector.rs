use crate::ast::node::{self, Node, Program};
use crate::ast::{Parser, Visitor};
use crate::token::Token;
// use crate::visitors::Visualizer;
use crate::error::InspectorError;

use log::{debug, info};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

type Result = std::result::Result<(), InspectorError>;

/// Inspector format
#[derive(Clone, Copy)]
pub enum InspectorMode {
    /// Short format.
    ///
    /// {name}({param, ...}): "{description}"
    Short,
    /// Long format.
    ///
    /// {name}: "{description}"
    ///
    /// path: {path}
    ///
    /// parameters:
    ///     {param}: {default}
    ///     ...
    Long,
    /// Dot format. As Long format, but also visualizes AST.
    Dot,
}

/// Walks AST and checks for symbols.
pub struct Inspector<'a> {
    name: String,
    path: PathBuf,
    description: String,
    parameters: Vec<(String, Option<String>)>,
    program: &'a Program,
    mode: InspectorMode,
}

impl<'a> Inspector<'a> {
    /// Public function for Inspector
    pub fn inspect<P: AsRef<Path>>(path: P, mode: InspectorMode) -> Result {
        let path = path.as_ref();
        let input_text =
            crate::normalize_newlines(&std::fs::read_to_string(path)?);

        let program = Parser::new(&input_text)?.parse()?;

        let mut inspector = Inspector {
            name: String::new(),
            path: dunce::canonicalize(path)?,
            description: String::new(),
            parameters: Vec::new(),
            program: &program,
            mode,
        };

        inspector.program.accept(&mut inspector);

        info!(r#"Inspected script "{}""#, inspector.name);

        let s = inspector.to_string();
        println!("{}", s);
        debug!("{}", s);

        Ok(())
    }

    fn fmt_short(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.parameters
                .iter()
                .map(|s| s.0.clone())
                .collect::<Vec<String>>()
                .join(", ")
        )?;

        if !self.description.is_empty() {
            write!(f, r#": "{}""#, self.description)?;
        }

        Ok(())
    }

    fn fmt_long(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_short(f)?;
        write!(f, "\n\npath: {}\n", self.path.display())?;
        if !self.parameters.is_empty() {
            write!(f, "\nparameters:")?;
            for param in &self.parameters {
                write!(f, "\n\t{}", param.0)?;
                if let Some(default) = &param.1 {
                    write!(f, r#": "{}""#, default)?;
                }
            }
        }
        Ok(())
    }

    fn fmt_dot(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // FIXME Construct dot-file here, return it to cli for rendering?

        // self.fmt_long(f)?;

        // let path = helpers::get_log_dir();

        // let dot =
        //     Visualizer::visualize_ast(self.program, &path, &self.name, true);

        // match dot {
        //     Ok(()) => write!(
        //         f,
        //         "\n\nRendered Abstract Syntax Tree to {}",
        //         path.join(&format!("{}.png", self.name)).display()
        //     ),
        //     Err(err) => write!(f, "\n\n{}", err),
        // }?;

        Ok(())
    }
}

impl<'a> Display for Inspector<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.mode {
            InspectorMode::Short => self.fmt_short(f),
            InspectorMode::Long => self.fmt_long(f),
            InspectorMode::Dot => self.fmt_dot(f),
        }
    }
}

impl<'a> Visitor<()> for Inspector<'a> {
    fn visit_program(&mut self, program: &node::Program) {
        self.name = program.name.get_string_unchecked().to_string();

        if let Some(description) = &program.description {
            self.description = description.get_string_unchecked().to_string();
        }

        program.parameters.accept(self);
        program.block.accept(self);
    }

    fn visit_parameters(&mut self, parameters: &node::Parameters) {
        parameters.parameters.iter().for_each(|e| e.accept(self));
    }

    fn visit_parameter(&mut self, parameter: &node::Parameter) {
        let name = parameter.token.get_string_unchecked().to_string();

        let default = parameter
            .default
            .as_ref()
            .map(|d| d.get_string_unchecked().to_string());

        self.parameters.push((name, default));
    }

    fn visit_block(&mut self, block: &node::Block) {
        block.expressions.iter().for_each(|e| e.accept(self));
    }

    fn visit_ternaryop(
        &mut self,
        condition: &node::Expression,
        true_expr: &node::Expression,
        false_expr: &node::Expression,
    ) {
        condition.accept(self);
        true_expr.accept(self);
        false_expr.accept(self);
    }

    fn visit_binaryop(
        &mut self,
        left: &node::Expression,
        _token: &Token,
        right: &node::Expression,
    ) {
        left.accept(self);
        right.accept(self);
    }

    fn visit_unaryop(&mut self, _token: &Token, operand: &node::Expression) {
        operand.accept(self);
    }

    fn visit_group(&mut self, expressions: &[node::Expression]) {
        expressions.iter().for_each(|e| e.accept(self));
    }

    fn visit_function(
        &mut self,
        _start_token: &Token,
        arguments: &[node::Expression],
    ) {
        arguments.iter().for_each(|e| e.accept(self));
    }

    fn visit_integer(&mut self, _integer: &Token) {}

    fn visit_string(&mut self, _string: &Token) {}

    fn visit_symbol(&mut self, _symbol: &Token) {}

    fn visit_tag(&mut self, _token: &Token) {}
}

// TODO Write test after config::read_script is implemented.
