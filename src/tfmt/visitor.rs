use crate::tfmt::ast::*;

/// [Visitor] visits [Node]s and returns `T`.
pub trait Visitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_parameters(&mut self, parameters: &Parameters) -> T;
    fn visit_parameter(&mut self, parameter: &Parameter) -> T;
    fn visit_block(&mut self, block: &Block) -> T;
    fn visit_driveletter(&mut self, driveletter: &DriveLetter) -> T;
    fn visit_expression(&mut self, expression: &Expression) -> T;
}
