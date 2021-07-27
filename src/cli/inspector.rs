use crate::cli::config;
use crate::tfmt::ast::{self, Node};
use crate::tfmt::token::Token;
use crate::tfmt::visitor::Visitor;
use anyhow::bail;

type Result = anyhow::Result<()>;

/// Walks AST and checks for symbols.
pub struct Inspector<'a> {
    path: &'a str,
}

impl<'a> Inspector<'a> {
    /// Public function for Inspector
    pub fn inspect(name: &str) -> Result {
        let (path, program) = config::read_script(name)?;

        program.accept(&mut Inspector {
            path: match &path.canonicalize()?.to_str() {
                Some(s) => s,
                None => bail!(
                    "Unable to convert path {:?} to valid unicode!",
                    &path
                ),
            },
        });

        Ok(())
    }
}

impl<'a> Visitor<()> for Inspector<'a> {
    fn visit_program(&mut self, program: &ast::Program) {
        print!("{}", program.name.get_value_unchecked());

        if let Some(description) = &program.description {
            print!(": \"{}\"", description.get_value_unchecked());
        }
        println!();

        println!("\npath: {}\n", self.path);

        program.parameters.accept(self);
        program.block.accept(self);
    }

    fn visit_parameters(&mut self, parameters: &ast::Parameters) {
        if !parameters.parameters.is_empty() {
            println!("parameters:");
        }

        parameters.parameters.iter().for_each(|e| e.accept(self));
    }

    fn visit_parameter(&mut self, parameter: &ast::Parameter) {
        print!("\t{}", parameter.token.get_value_unchecked());

        if let Some(default) = &parameter.default {
            print!(" = \"{}\"", default.get_value_unchecked());
        }

        println!();
    }

    fn visit_block(&mut self, block: &ast::Block) {
        block.expressions.iter().for_each(|e| e.accept(self));
    }

    fn visit_ternaryop(
        &mut self,
        condition: &ast::Expression,
        true_expr: &ast::Expression,
        false_expr: &ast::Expression,
    ) {
        condition.accept(self);
        true_expr.accept(self);
        false_expr.accept(self);
    }

    fn visit_binaryop(
        &mut self,
        left: &ast::Expression,
        _token: &Token,
        right: &ast::Expression,
    ) {
        left.accept(self);
        right.accept(self);
    }

    fn visit_unaryop(&mut self, _token: &Token, operand: &ast::Expression) {
        operand.accept(self);
    }

    fn visit_group(&mut self, expressions: &[ast::Expression]) {
        expressions.iter().for_each(|e| e.accept(self));
    }

    fn visit_function(
        &mut self,
        _start_token: &Token,
        arguments: &[ast::Expression],
    ) {
        arguments.iter().for_each(|e| e.accept(self));
    }

    fn visit_integer(&mut self, _integer: &Token) {}

    fn visit_string(&mut self, _string: &Token) {}

    fn visit_symbol(&mut self, _symbol: &Token) {}

    fn visit_tag(&mut self, _token: &Token) {}
}

// TODO Write test after config::read_script is implemented.
