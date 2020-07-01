use std::str::FromStr;
use std::error::Error;

use super::token::Token;

trait Node {
    fn token(&self) -> Token;
}

trait Literal<T: FromStr>: Node {
    fn value(&self) -> Result<T, T::Err> {
        // TODO Do something safer?
        self.token().value.expect("Literal must have a value!").parse::<T>()
    }
}

trait ID: Node {
    fn identifier(&self) -> &str {
        &self.token().value.expect("ID should always have a value!")
    }
}

trait Operator: Node {
    fn operator(&self) -> Token {
        self.token()
    }
}

pub struct Program {
    name: Token,
    parameters: Parameters,
    description: Option<Token>,
    block: Block,
}

impl Node for Program {
    fn token(&self) -> Token {
        self.name
    }
}

// FIXME Needs to be a node?
pub struct Parameters {
    parameters: Vec<Parameter>,
}

pub struct Parameter {
    token: Token,
    default: Option<Token>,
}

impl Node for Parameter {
    fn token(&self) -> Token {
        self.token
    }
}
impl ID for Parameter {}

// FIXME Needs to be a node?
pub struct Block {
    drive: Option<DriveLetter>,
    expression: Vec<Box<dyn Node>>,
}

// FIXME Needs to be a node?
pub struct TernaryOp {
    condition: Box<dyn Node>,
    true_expr: Box<dyn Node>,
    false_expr: Box<dyn Node>,
}

pub struct BinOp {
    left: Box<dyn Node>,
    token: Token,
    right: Box<dyn Node>,
}

impl Node for BinOp {
    fn token(&self) -> Token {
        self.token
    }
}
impl Operator for BinOp {}

pub struct UnaryOp {
    token: Token,
    operand: Box<dyn Node>,
}

impl Node for UnaryOp {
    fn token(&self) -> Token {
        self.token
    }
}
impl Operator for UnaryOp {}

// FIXME Needs to be a node?
pub struct Group {
    expression: Vec<Box<dyn Node>>,
}

pub struct Function {
    start_token: Token,
    Arguments: Vec<Box<dyn Node>>,
    end_token: Token
}
impl Node for Function {
    fn token(&self) -> Token {
        self.start_token
    }
}
impl ID for UnaryOp {}

#[derive(Debug)]
pub struct Integer {
    token: Token
}

impl Node for Integer {
    fn token(&self) -> Token {
        self.token
    }
}

impl Literal<u32> for Integer {}

//FIXME rest of AST
