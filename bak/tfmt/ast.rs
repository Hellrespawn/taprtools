use super::token::Token;

// Node visitor

pub trait Visitor {
    fn visit_program(&mut self, program: &Program);
    fn visit_parameters(&mut self, parameters: &Parameters);
    fn visit_parameter(&mut self, parameter: &Parameter);
    fn visit_block(&mut self, block: &Block);
    fn visit_ternaryop(&mut self, ternaryop: &TernaryOp);
    fn visit_binaryop(&mut self, binaryop: &BinaryOp);
    fn visit_unaryop(&mut self, unaryop: &UnaryOp);
    fn visit_group(&mut self, group: &Group);
    fn visit_function(&mut self, function: &Function);
    fn visit_integer(&mut self, integer: &Integer);
    fn visit_stringnode(&mut self, stringnode: &StringNode);
    fn visit_substitution(&mut self, substitution: &Substitution);
    fn visit_driveletter(&mut self, driveletter: &DriveLetter);
    fn visit_tag(&mut self, tag: &Tag);
}

// struct StringVisitor {}

// impl Visitor for StringVisitor {
//     type Result = String;
//     fn visit_foo(&self, foo: &Foo) -> String {
//         format!("it was Foo: {:}!", foo.value)
//     }
//     fn visit_bar(&self, bar: &Bar) -> String {
//         format!("it was Bar: {:}!", bar.value)
//     }
// }

// Hierarchy of traits
pub trait Node {
    fn accept(&self, v: &mut dyn Visitor);
}

// Program
pub struct Program {
    pub name: Token,
    pub parameters: Parameters,
    pub description: Option<Token>,
    pub block: Block,
}

impl Node for Program {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_program(&self)
    }
}

// Parameters
pub struct Parameters {
    pub parameters: Vec<Parameter>,
}
impl Node for Parameters {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_parameters(&self)
    }
}

// Parameter
pub struct Parameter {
    pub token: Token,
    // TODO? Token here is inconsistent, should be INT/STR.
    pub default: Option<Token>,
}
impl Node for Parameter {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_parameter(&self)
    }
}

// Block
pub struct Block {
    pub drive: Option<DriveLetter>,
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Block {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_block(&self)
    }
}

// TernaryOp
pub struct TernaryOp {
    pub condition: Box<dyn Node>,
    pub true_expr: Box<dyn Node>,
    pub false_expr: Box<dyn Node>,
}
impl Node for TernaryOp {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_ternaryop(&self)
    }
}

// BinOp
pub struct BinaryOp {
    pub left: Box<dyn Node>,
    pub token: Token,
    pub right: Box<dyn Node>,
}
impl Node for BinaryOp {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_binaryop(&self)
    }
}

// UnaryOp
pub struct UnaryOp {
    pub token: Token,
    pub operand: Box<dyn Node>,
}
impl Node for UnaryOp {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_unaryop(&self)
    }
}

// Group
pub struct Group {
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Group {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_group(&self)
    }
}

// Function
pub struct Function {
    pub start_token: Token,
    pub arguments: Vec<Box<dyn Node>>,
    pub end_token: Token,
}
impl Node for Function {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_function(&self)
    }
}

// Integer
pub struct Integer {
    pub token: Token,
}
impl Node for Integer {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_integer(&self)
    }
}

// String
pub struct StringNode {
    pub token: Token,
}
impl Node for StringNode {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_stringnode(&self)
    }
}

// String
pub struct Substitution {
    pub token: Token,
}

impl Node for Substitution {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_substitution(&self)
    }
}

// DriveLetter
pub struct DriveLetter {
    pub token: Token,
}

impl Node for DriveLetter {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_driveletter(&self)
    }
}

// Tag
pub struct Tag {
    pub start_token: Token,
    pub token: Token,
}
impl Node for Tag {
    fn accept(&self, v: &mut dyn Visitor) {
        v.visit_tag(&self)
    }
}
