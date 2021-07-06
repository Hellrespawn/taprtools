use super::token::Token;
use super::visitor::Visitor;

pub trait Node: std::fmt::Debug {
    fn accept(&self, visitor: &mut dyn Visitor);
}

#[derive(Debug)]
pub struct Program {
    pub name: Token,
    pub parameters: Parameters,
    pub description: Option<Token>,
    pub block: Block,
}

impl Node for Program {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_program(self)
    }
}

#[derive(Debug)]
pub struct Parameters {
    pub parameters: Vec<Parameter>,
}

impl Node for Parameters {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_parameters(self)
    }
}

#[derive(Debug)]
pub struct Parameter {
    pub token: Token,
    pub default: Option<Token>,
}

impl Node for Parameter {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_parameter(self)
    }
}

#[derive(Debug)]
pub struct Block {
    pub drive: Option<DriveLetter>,
    pub expressions: Vec<Box<dyn Node>>,
}

impl Node for Block {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_block(self)
    }
}

#[derive(Debug)]
pub struct TernaryOp {
    pub condition: Box<dyn Node>,
    pub true_expr: Box<dyn Node>,
    pub false_expr: Box<dyn Node>,
}
impl Node for TernaryOp {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_ternaryop(&self)
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub left: Box<dyn Node>,
    pub token: Token,
    pub right: Box<dyn Node>,
}
impl Node for BinaryOp {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_binaryop(&self)
    }
}

#[derive(Debug)]
pub struct UnaryOp {
    pub token: Token,
    pub operand: Box<dyn Node>,
}
impl Node for UnaryOp {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_unaryop(&self)
    }
}

#[derive(Debug)]
pub struct Group {
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Group {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_group(&self)
    }
}

#[derive(Debug)]
pub struct Function {
    pub start_token: Token,
    pub arguments: Vec<Box<dyn Node>>,
    pub end_token: Token,
}
impl Node for Function {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_function(&self)
    }
}

#[derive(Debug)]
pub struct StringNode {
    pub string: Token,
}

impl Node for StringNode {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_string(self)
    }
}

#[derive(Debug)]
pub struct IntegerNode {
    pub integer: Token,
}

impl Node for IntegerNode {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_integer(self)
    }
}

#[derive(Debug)]
pub struct Substitution {
    pub token: Token,
}

impl Node for Substitution {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_substitution(&self)
    }
}

#[derive(Debug)]
pub struct DriveLetter {
    pub token: Token,
}

impl Node for DriveLetter {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_driveletter(&self)
    }
}

#[derive(Debug)]
pub struct Tag {
    pub start_token: Token,
    pub token: Token,
}
impl Node for Tag {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_tag(&self)
    }
}
