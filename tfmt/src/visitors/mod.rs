///Interpreter
pub(crate) mod interpreter;
/// Semantic Analysis
pub(crate) mod semantic;

pub use interpreter::Interpreter;
pub(crate) use semantic::{ScriptParameter, SemanticAnalyzer};
