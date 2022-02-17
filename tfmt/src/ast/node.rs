#![allow(missing_docs)]
use crate::ast::visitor::Visitor;
use crate::token::Token;

/// [Node] accepts a [Visitor], according to the [Visitor pattern].
///
/// [Visitor pattern]: https://en.wikipedia.org/wiki/Visitor_pattern
pub(crate) trait Node<T>: std::fmt::Debug {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T;
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub(crate) name: Token,
    pub(crate) parameters: Parameters,
    pub(crate) description: Option<Token>,
    pub(crate) block: Block,
}

impl<T> Node<T> for Program {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_program(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameters {
    pub(crate) parameters: Vec<Parameter>,
}

impl<T> Node<T> for Parameters {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_parameters(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    pub(crate) token: Token,
    pub(crate) default: Option<Token>,
}

impl<T> Node<T> for Parameter {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_parameter(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub(crate) expressions: Vec<Expression>,
}

impl<T> Node<T> for Block {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_block(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    TernaryOp {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: Token,
        operand: Box<Expression>,
    },
    Group {
        expressions: Vec<Expression>,
    },
    Function {
        start_token: Token,
        arguments: Vec<Expression>,
        end_token: Token,
    },
    StringNode(Token),
    IntegerNode(Token),
    Symbol(Token),
    Tag {
        start_token: Token,
        token: Token,
    },
}

impl<T> Node<T> for Expression {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_expression(self)
    }
}
