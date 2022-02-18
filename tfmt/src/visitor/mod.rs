pub(crate) mod dot;
pub(crate) mod semantic;

mod interpreter;
mod visitor_trait;

pub(crate) use visitor_trait::Visitor;

pub use interpreter::Interpreter;
