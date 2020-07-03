use super::ast;

use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GenAstDot {
    root: ast::Program,
    node_count: u32,
    dot_body: Vec<String>,
}

pub fn visualize_ast(
    root: ast::Program,
    directory: &Path,
    name: &str,
    remove_dot_file: bool,
) -> Result<(), Box<dyn Error>> {
    let dot = GenAstDot::new(root).gen_ast_dot()?;

    let mut path = PathBuf::from(directory);
    path.push(format!("{}.dot", name));

    fs::create_dir_all(directory)?;

    let mut file = fs::File::create(&path)?;
    file.write_all(dot.as_bytes())?;

    let _output = Command::new("dot")
        .arg("-Tpng")
        .arg(format!("-o {:?}.png", name))
        .arg(format!("{:?}.dot", name));

    if remove_dot_file {
        fs::remove_file(path)?;
    }

    Ok(())
}

impl GenAstDot {
    pub fn new(root: ast::Program) -> GenAstDot {
        GenAstDot {
            root,
            node_count: 0,
            dot_body: Vec::new(),
        }
    }

    fn gen_ast_dot(&self) -> Result<String, Box<dyn Error>> {
        Ok("temp".to_owned())
    }
}

impl ast::Visitor for GenAstDot {
    //type Result;
    fn visit_program(&self, program: &ast::Program) {

    }
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
