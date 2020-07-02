use std::str::FromStr;

use super::token::Token;

// Hierarchy of traits
pub trait Node {}

pub trait HasToken: Node {
    fn token(&self) -> &Token;
}

pub trait Literal<T: FromStr>: HasToken {
    fn value(&self) -> Result<T, T::Err> {
        // TODO Do something safer?
        self.token()
            .value
            .as_ref()
            .expect("Literal must have a value!")
            .parse::<T>()
    }
}

pub trait ID: HasToken {
    fn identifier(&self) -> &str {
        &self
            .token()
            .value
            .as_ref()
            .expect("ID should always have a value!")
    }
}

pub trait Operator: HasToken {
    fn operator(&self) -> &Token {
        &self.token()
    }
}

// Program
pub struct Program {
    pub name: Token,
    pub parameters: Parameters,
    pub description: Option<Token>,
    pub block: Block,
}

impl Node for Program {}

impl HasToken for Program {
    fn token(&self) -> &Token {
        &self.name
    }
}

// Parameters
pub struct Parameters {
    pub parameters: Vec<Parameter>,
}
impl Node for Parameters {}

// Parameter
pub struct Parameter {
    pub token: Token,
    pub default: Option<Token>,
}
impl Node for Parameter {}

impl HasToken for Parameter {
    fn token(&self) -> &Token {
        &self.token
    }
}
impl ID for Parameter {}

// Block
pub struct Block {
    pub drive: Option<DriveLetter>,
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Block {}

// TernaryOp
pub struct TernaryOp {
    pub condition: Box<dyn Node>,
    pub true_expr: Box<dyn Node>,
    pub false_expr: Box<dyn Node>,
}
impl Node for TernaryOp {}

// BinOp
pub struct BinOp {
    pub left: Box<dyn Node>,
    pub token: Token,
    pub right: Box<dyn Node>,
}
impl Node for BinOp {}

impl HasToken for BinOp {
    fn token(&self) -> &Token {
        &self.token
    }
}
impl Operator for BinOp {}

// UnaryOp
pub struct UnaryOp {
    token: Token,
    operand: Box<dyn HasToken>,
}
impl Node for UnaryOp {}

impl HasToken for UnaryOp {
    fn token(&self) -> &Token {
        &self.token
    }
}
impl Operator for UnaryOp {}

// Group
pub struct Group {
    pub expression: Vec<Box<dyn Node>>,
}
impl Node for Group {}

// Function
pub struct Function {
    pub start_token: Token,
    pub arguments: Vec<Box<dyn Node>>,
    pub end_token: Token,
}
impl Node for Function {}

impl HasToken for Function {
    fn token(&self) -> &Token {
        &self.start_token
    }
}
impl ID for Function {}

// Integer
pub struct Integer {
    pub token: Token,
}

impl Node for Integer {}
impl HasToken for Integer {
    fn token(&self) -> &Token {
        &self.token
    }
}

impl Literal<u32> for Integer {}

// String
pub struct StringNode {
    pub token: Token,
}

impl Node for StringNode {}
impl HasToken for StringNode {
    fn token(&self) -> &Token {
        &self.token
    }
}

impl Literal<String> for StringNode {}

// String
pub struct Substitution {
    pub token: Token,
}

impl Node for Substitution {}
impl HasToken for Substitution {
    fn token(&self) -> &Token {
        &self.token
    }
}

impl Literal<String> for Substitution {}

// DriveLetter
pub struct DriveLetter {
    pub token: Token,
}

impl Node for DriveLetter {}
impl HasToken for DriveLetter {
    fn token(&self) -> &Token {
        &self.token
    }
}

impl Literal<String> for DriveLetter {}

// Tag
pub struct Tag {
    pub start_token: Token,
    pub end_token: Token,
}
impl Node for Tag {}
impl HasToken for Tag {
    fn token(&self) -> &Token {
        &self.start_token
    }
}

impl ID for Tag {}
