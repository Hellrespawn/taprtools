#![allow(missing_docs)]

#[allow(clippy::wildcard_imports)]
use crate::ast::node::*;
use crate::token::Token;

/// [Visitor] visits [Node]s and returns `T`.
pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;

    fn visit_parameters(&mut self, parameters: &Parameters) -> T;

    fn visit_parameter(&mut self, parameter: &Parameter) -> T;

    fn visit_block(&mut self, block: &Block) -> T;

    fn visit_expression(&mut self, expression: &Expression) -> T {
        match expression {
            Expression::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => self.visit_ternaryop(condition, true_expr, false_expr),
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => self.visit_binaryop(left, operator, right),
            Expression::UnaryOp { operator, operand } => {
                self.visit_unaryop(operator, operand)
            }
            Expression::Group { expressions } => self.visit_group(expressions),
            Expression::Function {
                start_token,
                arguments,
                ..
            } => self.visit_function(start_token, arguments),
            Expression::StringNode(string) => self.visit_string(string),
            Expression::IntegerNode(integer) => self.visit_integer(integer),
            Expression::Symbol(subst) => self.visit_symbol(subst),
            Expression::Tag { token, .. } => self.visit_tag(token),
        }
    }

    fn visit_ternaryop(
        &mut self,
        condition: &Expression,
        true_expr: &Expression,
        false_expr: &Expression,
    ) -> T;

    fn visit_binaryop(
        &mut self,
        left: &Expression,
        operator: &Token,
        right: &Expression,
    ) -> T;

    fn visit_unaryop(&mut self, token: &Token, operand: &Expression) -> T;

    fn visit_group(&mut self, expressions: &[Expression]) -> T;

    fn visit_function(
        &mut self,
        start_token: &Token,
        arguments: &[Expression],
    ) -> T;

    fn visit_integer(&mut self, integer: &Token) -> T;

    fn visit_string(&mut self, string: &Token) -> T;

    fn visit_symbol(&mut self, symbol: &Token) -> T;

    fn visit_tag(&mut self, token: &Token) -> T;
}
