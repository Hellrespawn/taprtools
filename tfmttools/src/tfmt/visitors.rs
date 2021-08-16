///Interpreter
pub mod interpreter;
/// Semantic Analysis
pub mod semantic;
/// AST Dot Generator
pub mod visualizer;

pub use interpreter::Interpreter;
pub use semantic::{SemanticAnalyzer, SymbolTable};
pub use visualizer::Visualizer;
