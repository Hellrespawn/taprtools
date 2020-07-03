use super::token::Token;

// Node visitor

pub trait Visitor {
    //type Result;
    fn visit_program(&self, program: &Program); // -> Self::Result;
    fn visit_parameters(&self, parameters: &Parameters); // -> Self::Result;
    fn visit_parameter(&self, parameter: &Parameter); // -> Self::Result;
    fn visit_block(&self, block: &Block); // -> Self::Result;
    fn visit_ternaryop(&self, ternaryop: &TernaryOp); // -> Self::Result;
    fn visit_binaryop(&self, binaryop: &BinaryOp); // -> Self::Result;
    fn visit_unaryop(&self, unaryop: &UnaryOp); // -> Self::Result;
    fn visit_group(&self, group: &Group); // -> Self::Result;
    fn visit_function(&self, function: &Function); // -> Self::Result;
    fn visit_integer(&self, integer: &Integer); // -> Self::Result;
    fn visit_stringnode(&self, stringnode: &StringNode); // -> Self::Result;
    fn visit_substitution(&self, substitution: &Substitution); // -> Self::Result;
    fn visit_driveletter(&self, driveletter: &DriveLetter); // -> Self::Result;
    fn visit_tag(&self, tag: &Tag); // -> Self::Result;
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
    fn accept(&self, v: &dyn Visitor);
}

// pub trait HasToken {
//     fn token(&self) -> &Token;
// }

// pub trait Literal<T: FromStr>: HasToken {
//     fn value(&self) -> Result<T, T::Err> {
//         // TODO Do something safer?
//         self.token()
//             .value
//             .as_ref()
//             .expect("Literal must have a value!")
//             .parse::<T>()
//     }
// }

// pub trait ID: HasToken {
//     fn identifier(&self) -> &str {
//         &self
//             .token()
//             .value
//             .as_ref()
//             .expect("ID should always have a value!")
//     }
// }

// pub trait Operator: HasToken {
//     fn operator(&self) -> &Token {
//         &self.token()
//     }
// }

// Program
pub struct Program {
    pub name: Token,
    pub parameters: Parameters,
    pub description: Option<Token>,
    pub block: Block,
}

impl Node for Program {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_program(&self)
    }
}

// impl HasToken for Program {
//     fn token(&self) -> &Token {
//         &self.name
//     }
// }

// Parameters
pub struct Parameters {
    pub parameters: Vec<Parameter>,
}
impl Node for Parameters {
    fn accept(&self, v: &dyn Visitor) {
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
    fn accept(&self, v: &dyn Visitor) {
        v.visit_parameter(&self)
    }
}

// impl HasToken for Parameter {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }
// impl ID for Parameter {}

// Block
pub struct Block {
    pub drive: Option<DriveLetter>,
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Block {
    fn accept(&self, v: &dyn Visitor) {
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
    fn accept(&self, v: &dyn Visitor) {
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
    fn accept(&self, v: &dyn Visitor) {
        v.visit_binaryop(&self)
    }
}

// impl HasToken for BinOp {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }
// impl Operator for BinOp {}

// UnaryOp
pub struct UnaryOp {
    pub token: Token,
    pub operand: Box<dyn Node>,
}
impl Node for UnaryOp {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_unaryop(&self)
    }
}

// impl HasToken for UnaryOp {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }
// impl Operator for UnaryOp {}

// Group
pub struct Group {
    pub expressions: Vec<Box<dyn Node>>,
}
impl Node for Group {
    fn accept(&self, v: &dyn Visitor) {
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
    fn accept(&self, v: &dyn Visitor) {
        v.visit_function(&self)
    }
}

// impl HasToken for Function {
//     fn token(&self) -> &Token {
//         &self.start_token
//     }
// }
// impl ID for Function {}

// Integer
pub struct Integer {
    pub token: Token,
}
impl Node for Integer {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_integer(&self)
    }
}
// impl HasToken for Integer {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }

// impl Literal<u32> for Integer {}

// String
pub struct StringNode {
    pub token: Token,
}
impl Node for StringNode {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_stringnode(&self)
    }
}
// impl HasToken for StringNode {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }

// impl Literal<String> for StringNode {}

// String
pub struct Substitution {
    pub token: Token,
}

impl Node for Substitution {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_substitution(&self)
    }
}
// impl HasToken for Substitution {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }

// impl Literal<String> for Substitution {}

// DriveLetter
pub struct DriveLetter {
    pub token: Token,
}

impl Node for DriveLetter {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_driveletter(&self)
    }
}
// impl HasToken for DriveLetter {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }

// impl Literal<String> for DriveLetter {}

// Tag
pub struct Tag {
    pub start_token: Token,
    pub token: Token,
}
impl Node for Tag {
    fn accept(&self, v: &dyn Visitor) {
        v.visit_tag(&self)
    }
}
// impl HasToken for Tag {
//     fn token(&self) -> &Token {
//         &self.token
//     }
// }

// impl ID for Tag {}
