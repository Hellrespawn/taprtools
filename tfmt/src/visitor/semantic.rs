use crate::ast::node::{self, Node};
use crate::error::SemanticError;
use crate::script::ScriptParameter;
use crate::token::Token;
use crate::visitor::Visitor;
use std::collections::HashMap;

pub(crate) struct Analysis {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) parameters: Vec<ScriptParameter>,
}

/// Walks AST and checks for symbols.
#[derive(Default)]
pub(crate) struct SemanticAnalyzer {
    name: String,
    description: Option<String>,
    parameters: HashMap<String, ScriptParameter>,
}

impl SemanticAnalyzer {
    /// Public function for [`SemanticAnalyzer`]
    pub(crate) fn analyze(
        program: &node::Program,
    ) -> Result<Analysis, SemanticError> {
        let mut analyzer: Self = SemanticAnalyzer::default();

        program.accept(&mut analyzer);

        // Check that all parameter occur in the program.
        for param in analyzer.parameters.values_mut() {
            if (*param.count()) == 0 {
                return Err(SemanticError::SymbolNotUsed(
                    param.name().to_string(),
                ));
            }
        }

        Ok(Analysis {
            name: analyzer.name,
            description: analyzer.description,
            parameters: analyzer.parameters.into_values().collect(),
        })
    }
}

impl Visitor<()> for SemanticAnalyzer {
    fn visit_program(&mut self, program: &node::Program) {
        self.name = program.name();
        self.description = program.description();

        program.parameters().accept(self);
        program.block().accept(self);
    }

    fn visit_parameters(&mut self, parameters: &node::Parameters) {
        parameters.parameters().iter().for_each(|e| e.accept(self));
    }

    fn visit_parameter(&mut self, parameter: &node::Parameter) {
        let name = parameter.name();

        let default = parameter.default();

        let param = ScriptParameter::new(name.clone(), default);

        self.parameters.insert(name, param);
    }

    fn visit_block(&mut self, block: &node::Block) {
        block.expressions().iter().for_each(|e| e.accept(self));
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
        let name = symbol.get_string_unchecked().to_string();

        self.parameters
            .entry(name)
            .and_modify(|p| (*p.count()) += 1);
    }

    fn visit_tag(&mut self, _token: &Token) {}
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::ast::{node, Parser};
//     use anyhow::Result;

//     fn get_script(path: &str) -> Result<node::Program> {
//         let input_text = crate::normalize_newlines(&std::fs::read_to_string(
//             format!("testdata/script/{}", path),
//         )?);

//         Ok(Parser::new(&input_text)?.parse()?)
//     }

//     fn script_test(name: &str, reference: &SymbolTable) -> Result<()> {
//         let program = get_script(name)?;

//         let symbol_table = SemanticAnalyzer::analyze(&program, &[])?;

//         assert_eq!(&symbol_table, reference);

//         Ok(())
//     }

//     #[test]
//     fn semantic_typical_input_test() -> Result<()> {
//         let mut map = HashMap::new();
//         map.insert("folder".to_string(), "destination".to_string());

//         script_test("typical_input.tfmt", &map)
//     }
// }
