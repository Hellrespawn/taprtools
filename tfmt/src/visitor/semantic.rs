use crate::ast::node::{self, Node};
use crate::error::SemanticError;
use crate::script::ScriptParameter;
use crate::token::Token;
use crate::visitor::Visitor;
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Analysis {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) parameters: Vec<ScriptParameter>,
}

type Result<T> = std::result::Result<T, SemanticError>;

/// Walks AST and checks for symbols.
#[derive(Default)]
pub(crate) struct SemanticAnalyzer {
    name: String,
    description: Option<String>,
    parameters: HashMap<String, ScriptParameter>,
}

impl SemanticAnalyzer {
    /// Public function for [`SemanticAnalyzer`]
    pub(crate) fn analyze(program: &node::Program) -> Result<Analysis> {
        let mut analyzer: Self = SemanticAnalyzer::default();

        program.accept(&mut analyzer)?;

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

impl Visitor<Result<()>> for SemanticAnalyzer {
    fn visit_program(&mut self, program: &node::Program) -> Result<()> {
        self.name = program.name();
        self.description = program.description();

        program.parameters().accept(self)?;
        program.block().accept(self)?;
        Ok(())
    }

    fn visit_parameters(
        &mut self,
        parameters: &node::Parameters,
    ) -> Result<()> {
        for parameter in parameters.parameters() {
            parameter.accept(self)?;
        }

        Ok(())
    }

    fn visit_parameter(&mut self, parameter: &node::Parameter) -> Result<()> {
        let name = parameter.name();

        let default = parameter.default();

        let param = ScriptParameter::new(name.clone(), default);

        self.parameters.insert(name, param);

        Ok(())
    }

    fn visit_block(&mut self, block: &node::Block) -> Result<()> {
        for expression in block.expressions() {
            expression.accept(self)?;
        }

        Ok(())
    }

    fn visit_ternaryop(
        &mut self,
        condition: &node::Expression,
        true_expr: &node::Expression,
        false_expr: &node::Expression,
    ) -> Result<()> {
        condition.accept(self)?;
        true_expr.accept(self)?;
        false_expr.accept(self)?;
        Ok(())
    }

    fn visit_binaryop(
        &mut self,
        left: &node::Expression,
        _token: &Token,
        right: &node::Expression,
    ) -> Result<()> {
        left.accept(self)?;
        right.accept(self)?;
        Ok(())
    }

    fn visit_unaryop(
        &mut self,
        _token: &Token,
        operand: &node::Expression,
    ) -> Result<()> {
        operand.accept(self)
    }

    fn visit_group(&mut self, expressions: &[node::Expression]) -> Result<()> {
        for expression in expressions {
            expression.accept(self)?;
        }

        Ok(())
    }

    fn visit_function(
        &mut self,
        _start_token: &Token,
        arguments: &[node::Expression],
    ) -> Result<()> {
        for expression in arguments {
            expression.accept(self)?;
        }
        Ok(())
    }

    fn visit_integer(&mut self, _integer: &Token) -> Result<()> {
        Ok(())
    }

    fn visit_string(&mut self, _string: &Token) -> Result<()> {
        Ok(())
    }

    fn visit_symbol(&mut self, symbol: &Token) -> Result<()> {
        let name = symbol.get_string_unchecked().to_string();

        if !self.parameters.contains_key(&name) {
            return Err(SemanticError::SymbolNotDeclared(name));
        }

        self.parameters
            .entry(name)
            .and_modify(|p| (*p.count()) += 1);

        Ok(())
    }

    fn visit_tag(&mut self, _token: &Token) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Parser;
    use anyhow::{bail, Result};

    #[test]
    fn test_analysis_full() -> Result<()> {
        let input_text = "typical_input(folder=\"destination\") \"This file is used to test tfmttools.\"{$(folder)}";
        let program = Parser::new(&input_text)?.parse()?;

        let mut analysis = SemanticAnalyzer::analyze(&program)?;

        assert_eq!(analysis.name, "typical_input");

        assert_eq!(
            analysis.description,
            Some("This file is used to test tfmttools.".to_string())
        );

        assert_eq!(analysis.parameters.len(), 1);
        let param = analysis.parameters.get_mut(0).unwrap();

        assert_eq!(param.name(), "folder");
        assert_eq!(param.default(), Some("destination"));
        assert_eq!((*param.count()), 1);

        Ok(())
    }

    #[test]
    fn test_analysis_param_not_used() -> Result<()> {
        let input_text = "typical_input(folder=\"destination\") \"This file is used to test tfmttools.\"{}";
        let program = Parser::new(&input_text)?.parse()?;

        let analysis = SemanticAnalyzer::analyze(&program);

        match analysis {
            Ok(_) => bail!("Expected SymbolNotUsed, got Ok(_)"),
            Err(err) => match err {
                SemanticError::SymbolNotUsed(symbol) => {
                    assert_eq!(symbol, "folder".to_string());
                    Ok(())
                }
                other => bail!("Expected SymbolNotUsed, got {}", other),
            },
        }
    }

    #[test]
    fn test_analysis_param_not_declared() -> Result<()> {
        let input_text = "typical_input() \"This file is used to test tfmttools.\"{$(folder)}";
        let program = Parser::new(&input_text)?.parse()?;

        let analysis = SemanticAnalyzer::analyze(&program);

        match analysis {
            Ok(_) => bail!("Expected SymbolNotDeclared, got Ok(_)"),
            Err(err) => match err {
                SemanticError::SymbolNotDeclared(symbol) => {
                    assert_eq!(symbol, "folder".to_string());
                    Ok(())
                }
                other => bail!("Expected SymbolNotDeclared, got {}", other),
            },
        }
    }
}
