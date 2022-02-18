///Interpreter
pub(crate) mod interpreter;
/// Semantic Analysis
pub(crate) mod semantic;
/// Visitor Trait
pub(crate) mod visitor;

pub(crate) use visitor::Visitor;

pub use interpreter::Interpreter;
