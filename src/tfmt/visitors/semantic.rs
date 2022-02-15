use crate::tfmt::ast::node::{self, Node};
use crate::tfmt::ast::Visitor;
use crate::tfmt::error::SemanticError;
use crate::tfmt::token::Token;
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
    /// Public function for [`SemanticAnalyzer`]
    pub fn analyze(
        program: &node::Program,
        arguments: &[&str],
    ) -> Result<SymbolTable, SemanticError> {
        let mut sa: Self = SemanticAnalyzer::default();

        program.accept(&mut sa);

        // Check that all parameter occur in the program.
        for (symbol, count) in sa.symbol_count {
            if count == 0 {
                return Err(SemanticError::SymbolNotUsed(symbol, sa.name));
            }
        }

        // Check that we have the right amount of arguments
        if arguments.len() > sa.symbols.len() {
            return Err(SemanticError::TooManyArguments {
                found: arguments.len(),
                expected: sa.symbols.len(),
                name: sa.name,
            });
        }

        let mut output = HashMap::new();

        for (symbol, default) in sa.symbols.iter().zip(sa.defaults) {
            output.insert(symbol, default);
        }

        for (symbol, argument) in sa.symbols.iter().zip(arguments) {
            // clippy::inefficient_to_string
            output.insert(symbol, Some((*argument).to_string()));
        }

        for (key, val) in &output {
            if val.is_none() {
                return Err(SemanticError::ArgumentRequired(
                    // clippy::inefficient_to_string
                    (*key).to_string(),
                    sa.name,
                ));
            }
        }
        let output = output
            .into_iter()
            // We tested for None, unwrap should be safe.
            .map(|(k, v)| (k.to_string(), v.unwrap()))
            .collect();

        info!("Symbol Table: {:?}", output);

        Ok(output)
    }
}

impl Visitor<()> for SemanticAnalyzer {
    fn visit_program(&mut self, program: &node::Program) {
        self.name = program.name.get_string_unchecked().to_string();

        program.parameters.accept(self);
        program.block.accept(self);
    }

    fn visit_parameters(&mut self, parameters: &node::Parameters) {
        parameters.parameters.iter().for_each(|e| e.accept(self));
    }

    fn visit_parameter(&mut self, parameter: &node::Parameter) {
        let key = parameter.token.get_string_unchecked();

        let default = parameter
            .default
            .as_ref()
            .map(|t| t.get_string_unchecked().to_string());

        self.symbols.push(key.to_string());
        self.symbol_count.insert(key.to_string(), 0);
        self.defaults.push(default);
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

    fn visit_symbol(&mut self, symbol: &Token) {
        let key = symbol.get_string_unchecked().to_string();

        self.symbol_count.entry(key).and_modify(|c| *c += 1);
    }

    fn visit_tag(&mut self, _token: &Token) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers;
    use crate::tfmt::ast::{node, Parser};
    use anyhow::Result;

    fn get_script(path: &str) -> Result<node::Program> {
        let input_text = helpers::normalize_newlines(&std::fs::read_to_string(
            format!("testdata/script/{}", path),
        )?);

        Ok(Parser::new(&input_text)?.parse()?)
    }

    fn script_test(name: &str, reference: &SymbolTable) -> Result<()> {
        let program = get_script(name)?;

        let symbol_table = SemanticAnalyzer::analyze(&program, &[])?;

        assert_eq!(&symbol_table, reference);

        Ok(())
    }

    #[test]
    fn semantic_typical_input_test() -> Result<()> {
        let mut map = HashMap::new();
        map.insert("folder".to_string(), "destination".to_string());

        script_test("typical_input.tfmt", &map)
    }
}
