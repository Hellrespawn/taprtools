pub mod ast {
    use super::super::token::Token;
    use super::visitor::*;

    pub trait Node {
        fn accept(&self, visitor: &mut dyn Visitor);
    }

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

    pub struct Parameters {
        pub parameters: Vec<Parameter>
    }

    impl Node for Parameters {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_parameters(self)
        }
    }

    pub struct Parameter {
        pub token: Token,
        pub default: Option<Token>
    }

    impl Node for Parameter {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_parameter(self)
        }
    }

    pub struct Block {
        pub drive: Option<DriveLetter>,
        pub expressions: Vec<Box<dyn Node>>,
    }

    impl Node for Block {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_block(self)
        }
    }

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

    pub struct UnaryOp {
        pub token: Token,
        pub operand: Box<dyn Node>,
    }
    impl Node for UnaryOp {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_unaryop(&self)
        }
    }

    pub struct Group {
        pub expressions: Vec<Box<dyn Node>>,
    }
    impl Node for Group {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_group(&self)
        }
    }

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

    pub struct StringNode {
        pub string: Token,
    }

    impl Node for StringNode {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_string(self)
        }
    }

    pub struct IntegerNode {
        pub integer: Token,
    }

    impl Node for IntegerNode {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_integer(self)
        }
    }

    pub struct Substitution {
        pub token: Token,
    }

    impl Node for Substitution {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_substitution(&self)
        }
    }

    pub struct DriveLetter {
        pub token: Token,
    }

    impl Node for DriveLetter {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_driveletter(&self)
        }
    }

    pub struct Tag {
        pub start_token: Token,
        pub token: Token,
    }
    impl Node for Tag {
        fn accept(&self, visitor: &mut dyn Visitor) {
            visitor.visit_tag(&self)
        }
    }

}

pub mod visitor {
    use super::ast::*;
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
        fn visit_integer(&mut self, integer: &IntegerNode);
        fn visit_string(&mut self, string: &StringNode);
        fn visit_substitution(&mut self, substitution: &Substitution);
        fn visit_driveletter(&mut self, driveletter: &DriveLetter);
        fn visit_tag(&mut self, tag: &Tag);
    }
}
