/// Inspector
pub(crate) mod inspector;
///Interpreter
pub(crate) mod interpreter;
/// Semantic Analysis
pub(crate) mod semantic;
/// AST Dot Generator
pub(crate) mod visualizer;

pub use inspector::{Inspector, InspectorMode};
pub use interpreter::Interpreter;
pub use visualizer::Visualizer;
pub(crate) use semantic::{SemanticAnalyzer, ScriptParameter};
