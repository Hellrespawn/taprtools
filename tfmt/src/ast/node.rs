#![allow(missing_docs)]
use crate::visitor::Visitor;
use crate::token::Token;

/// [Node] accepts a [Visitor], according to the [Visitor pattern].
///
/// [Visitor pattern]: https://en.wikipedia.org/wiki/Visitor_pattern
pub(crate) trait Node<T>: std::fmt::Debug {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T;
}

#[derive(Debug, PartialEq)]
pub struct Program {
    name: Token,
    parameters: Parameters,
    description: Option<Token>,
    block: Block,
}

impl<T> Node<T> for Program {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_program(self)
    }
}

impl Program {
    pub(crate) fn new(
        name: Token,
        parameters: Parameters,
        description: Option<Token>,
        block: Block,
    ) -> Self {
        Program {
            name,
            parameters,
            description,
            block,
        }
    }

    pub(crate) fn name(&self) -> String {
        self.name.get_string_unchecked().to_string()
    }

    pub(crate) fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    pub(crate) fn description(&self) -> Option<String> {
        self.description
            .as_ref()
            .map(|t| t.get_string_unchecked().to_string())
    }

    pub(crate) fn block(&self) -> &Block {
        &self.block
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameters {
    parameters: Vec<Parameter>,
}

impl Parameters {
    pub(crate) fn new(parameters: Vec<Parameter>) -> Self {
        Parameters { parameters }
    }

    pub(crate) fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
}

impl<T> Node<T> for Parameters {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_parameters(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameter {
    name: Token,
    default: Option<Token>,
}

impl Parameter {
    pub(crate) fn new(name: Token, default: Option<Token>) -> Self {
        Parameter { name, default }
    }

    pub(crate) fn name(&self) -> String {
        self.name.get_string_unchecked().to_string()
    }

    pub(crate) fn default(&self) -> Option<String> {
        self.default
            .as_ref()
            .map(|t| t.get_string_unchecked().to_string())
    }
}

impl<T> Node<T> for Parameter {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        visitor.visit_parameter(self)
    }
}

#[derive(Debug, PartialEq)]
pub struct Block {
    expressions: Vec<Expression>,
}

impl Block {
    pub(crate) fn new(expressions: Vec<Expression>) -> Self {
        Block { expressions }
    }

    pub(crate) fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
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
