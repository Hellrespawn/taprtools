use crate::tfmt::ast::*;

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
