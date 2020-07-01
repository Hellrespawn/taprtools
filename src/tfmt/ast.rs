use std::str::FromStr;

use super::token::Token;

// Hierarchy of traits
trait Node {}

trait HasToken: Node {
    fn token(&self) -> &Token;
}

trait Literal<T: FromStr>: HasToken {
    fn value(&self) -> Result<T, T::Err> {
        // TODO Do something safer?
        self.token()
            .value
            .as_ref()
            .expect("Literal must have a value!")
            .parse::<T>()
    }
}

trait ID: HasToken {
    fn identifier(&self) -> &str {
        &self
            .token()
            .value
            .as_ref()
            .expect("ID should always have a value!")
    }
}

trait Operator: HasToken {
    fn operator(&self) -> &Token {
        &self.token()
    }
}

// Program
pub struct Program {
    name: Token,
    parameters: Parameters,
    description: Option<Token>,
    block: Block,
}

impl Node for Program {}

impl HasToken for Program {
    fn token(&self) -> &Token {
        &self.name
    }
}

// Parameters
pub struct Parameters {
    parameters: Vec<Parameter>,
}
impl Node for Parameters {}

// Parameter
pub struct Parameter {
    token: Token,
    default: Option<Token>,
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
    drive: Option<Box<dyn Literal<String>>>,
    expression: Vec<Box<dyn HasToken>>,
}
impl Node for Block {}

// TernaryOp
pub struct TernaryOp {
    condition: Box<dyn HasToken>,
    true_expr: Box<dyn HasToken>,
    false_expr: Box<dyn HasToken>,
}
impl Node for TernaryOp {}

// BinOp
pub struct BinOp {
    left: Box<dyn HasToken>,
    token: Token,
    right: Box<dyn HasToken>,
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
    expression: Vec<Box<dyn HasToken>>,
}
impl Node for Group {}

// Function
pub struct Function {
    start_token: Token,
    arguments: Vec<Box<dyn HasToken>>,
    end_token: Token,
}
impl Node for Function {}

impl HasToken for Function {
    fn token(&self) -> &Token {
        &self.start_token
    }
}
impl ID for Function {}

pub struct LiteralStruct {
    token: Token,
}

impl Node for LiteralStruct {}
impl HasToken for LiteralStruct {
    fn token(&self) -> &Token {
        &self.token
    }
}

impl Literal<u32> for LiteralStruct {}
impl Literal<String> for LiteralStruct {}

// Tag
pub struct Tag {
    start_token: Token,
    end_token: Token,
}
impl Node for Tag {}
impl HasToken for Tag {
    fn token(&self) -> &Token {
        &self.start_token
    }
}

impl ID for Tag {}
