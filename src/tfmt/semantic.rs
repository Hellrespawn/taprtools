use super::ast::{self, Node};
use super::token::Token;
use super::visitor::Visitor;
use crate::error::SemanticError;

use log::info;

use std::collections::HashMap;

/// Holds symbols for [Interpreter].
pub type SymbolTable = HashMap<String, String>;

/// Walks AST and checks for symbols.
#[derive(Default)]
pub struct SemanticAnalyzer {
    name: String,
    symbols: Vec<String>,
    symbol_count: HashMap<String, u64>,
    defaults: Vec<Option<String>>,
}

impl SemanticAnalyzer {
    /// Public function for SemanticAnalyzer
    pub fn analyze(
        program: ast::Program,
        arguments: Vec<String>,
    ) -> Result<SymbolTable, SemanticError> {
        let mut sa: SemanticAnalyzer = Default::default();

        program.accept(&mut sa);

        // Check that all parameter occur in the program.
        for (symbol, count) in sa.symbol_count {
            if count == 0 {
                return Err(SemanticError::SymbolNotUsed(symbol, sa.name));
            }
        }

        // Check that we have the right amount of arguments
        if arguments.len() > sa.symbols.len() {
            return Err(SemanticError::TooManyArguments(
                arguments.len(),
                sa.symbols.len(),
                sa.name,
            ));
        }

        let mut output = HashMap::new();

        for (symbol, default) in sa.symbols.iter().zip(sa.defaults) {
            output.insert(symbol, default);
        }

        for (symbol, argument) in sa.symbols.iter().zip(arguments) {
            output.insert(symbol, Some(argument));
        }

        for (key, val) in &output {
            if val.is_none() {
                return Err(SemanticError::ArgumentRequired(
                    key.to_string(),
                    sa.name,
                ));
            }
        }
        let output = output
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.unwrap()))
            .collect();

        info!("Symbol Table: {:?}", output);

        // We tested for None, unwrap should be safe.
        Ok(output)
    }
}

impl Visitor<()> for SemanticAnalyzer {
    fn visit_program(&mut self, program: &ast::Program) {
        self.name = program.name.get_value_unchecked().to_string();

        program.parameters.accept(self);
        program.block.accept(self);
    }

    fn visit_parameters(&mut self, parameters: &ast::Parameters) {
        parameters.parameters.iter().for_each(|e| e.accept(self));
    }

    fn visit_parameter(&mut self, parameter: &ast::Parameter) {
        let key = parameter.token.get_value_unchecked();

        let default = parameter
            .default
            .as_ref()
            .map(|t| t.get_value_unchecked().to_string());

        self.symbols.push(key.to_string());
        self.symbol_count.insert(key.to_string(), 0);
        self.defaults.push(default);
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

    fn visit_symbol(&mut self, symbol: &Token) {
        let key = symbol.get_value_unchecked().to_string();

        self.symbol_count.entry(key).and_modify(|c| *c += 1);
    }

    fn visit_tag(&mut self, _token: &Token) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tfmt::ast;
    use crate::tfmt::parser::Parser;
    use anyhow::Result;
    use maplit::hashmap;
    use std::str::FromStr;

    fn get_script(path: &str) -> Result<ast::Program> {
        Ok(Parser::from_str(&std::fs::read_to_string(format!(
            "testdata/script/{}",
            path
        ))?)?
        .parse()?)
    }

    fn script_test(name: &str, reference: &SymbolTable) -> Result<()> {
        let program = get_script(name)?;

        let symbol_table = SemanticAnalyzer::analyze(program, Vec::new())?;

        assert_eq!(&symbol_table, reference);

        Ok(())
    }

    #[test]
    fn test_typical_input() -> Result<()> {
        script_test(
            "typical_input.tfmt",
            &hashmap! {
                "folder".to_string() => "destination".to_string()
            },
        )
    }
}
