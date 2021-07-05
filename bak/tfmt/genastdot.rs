use super::ast::{self, Node};
use super::token::TOKEN_TYPE_STRING_MAP;
use crate::error::TFMTError;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

pub struct GenAstDot<'a> {
    dot_body: &'a mut String,
    counter: u64,
}

pub fn visualize_ast(
    root: ast::Program,
    directory: &Path,
    name: &str,
    remove_dot_file: bool,
) -> Result<()> {
    let mut dot_body = String::new();

    let mut g = GenAstDot::new(&mut dot_body);

    root.accept(&mut g);

    let mut dot = "digraph astgraph {\n  \
        edge [arrowsize=.5];\n  \
        rankdir=\"TB\";\n  \
        newrank=true;\n  \
        nodesep=0.75;\n  \
        ranksep=0.75;\n  "
        .to_owned();

    dot.push_str(&dot_body);

    dot.push_str("}");

    let mut path = PathBuf::from(directory);
    path.push(format!("{}.dot", name));

    fs::create_dir_all(directory)?;

    let mut file = fs::File::create(&path)?;
    file.write_all(dot.as_bytes())?;

    let result = Command::new("dot")
        .current_dir(directory)
        .arg("-Tpng")
        .args(&["-o", &format!("{}.png", name)])
        .arg(format!("{}.dot", name))
        .spawn();

    if result.is_err() {
        return Err(TFMTError::GenAstDot.into());
    }

    if remove_dot_file {
        fs::remove_file(path)?;
    }

    Ok(())
}

impl<'a> GenAstDot<'a> {
    pub fn new(dot_body: &'a mut String) -> GenAstDot<'a> {
        GenAstDot {
            dot_body,
            counter: 0,
        }
    }

    fn increment(&mut self) -> u64 {
        self.counter += 1;
        self.counter - 1
    }

    fn create_node(&mut self, label: &str, hidden: bool) -> u64 {
        let label_str = {
            if hidden {
                "[shape=point]".to_owned()
            } else {
                format!("[label=\"{}\"]", label.replace("\n", "\\n"))
            }
        };

        self.dot_body
            .push_str(&format!("  node{} {}\n", self.counter, label_str));

        self.increment()
    }

    fn new_node(&mut self, label: &str) -> u64 {
        self.create_node(label, false)
    }

    fn hidden_node(&mut self) -> u64 {
        self.create_node("", true)
    }

    fn node_connector(
        &mut self,
        node1: u64,
        node2: u64,
        label: Option<&str>,
        directed: bool,
    ) {
        let mut string = format!("  node{} -> node{}", node2, node1);

        let mut args: Vec<String> = Vec::new();

        if let Some(label) = label {
            args.push(format!("label=\"{}\"", label.replace("\n", "\\n")));
        }

        if !directed {
            args.push("dir=none".to_owned())
        }

        if !args.is_empty() {
            string.push_str(&format!(" [{}]", args.join(", ")))
        }

        string.push_str("\n");

        self.dot_body.push_str(&string);
    }

    fn connect_nodes(&mut self, node1: u64, node2: u64) {
        self.node_connector(node1, node2, None, true)
    }

    fn connect_nodes_with_label(
        &mut self,
        node1: u64,
        node2: u64,
        label: &str,
    ) {
        self.node_connector(node1, node2, Some(label), true)
    }
}

impl<'a> ast::Visitor for GenAstDot<'a> {
    //type Result;
    fn visit_program(&mut self, program: &ast::Program) {
        let program_node = self.new_node(&format!(
            "Program\n{}",
            program
                .name
                .value
                .as_ref()
                .expect("Name required for program!")
        ));

        self.dot_body
            .push_str("subgraph header {\nrankdir=\"RL\";\n");

        let parameters_node = self.counter;
        program.parameters.accept(self);
        self.connect_nodes(parameters_node, program_node);

        self.dot_body
            .push_str("}\nsubgraph block  {\nrankdir=\"LR\";\n");

        let block_node = self.counter;
        program.block.accept(self);
        self.connect_nodes(block_node, program_node);

        self.dot_body.push_str("}\n");
    }

    fn visit_parameters(&mut self, parameters: &ast::Parameters) {
        let parameters_node = self
            .new_node(&format!("Params:\n({})", parameters.parameters.len()));

        for parameter in parameters.parameters.iter() {
            let parameter_node = self.counter;
            parameter.accept(self);
            self.connect_nodes(parameter_node, parameters_node);
        }
    }

    fn visit_parameter(&mut self, parameter: &ast::Parameter) {
        let parameter_node = self.new_node(
            parameter
                .token
                .value
                .as_ref()
                .expect("Parameter token must have value!"),
        );

        if let Some(default) = parameter.default.as_ref() {
            let default_node = self.new_node(
                default
                    .value
                    .as_ref()
                    .expect("Parameter default token must have value!"),
            );
            self.connect_nodes(default_node, parameter_node);
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        let block_node = self.new_node("Block");

        if let Some(drive) = &block.drive {
            let drive_node = self.counter;
            drive.accept(self);
            self.connect_nodes_with_label(
                drive_node,
                block_node,
                "drive letter",
            );
        }

        let expressions_node = self.hidden_node();
        self.node_connector(
            expressions_node,
            block_node,
            Some("expressions"),
            false,
        );

        for expression in block.expressions.iter() {
            let expression_node = self.counter;
            expression.accept(self);
            self.connect_nodes(expression_node, expressions_node)
        }
    }

    fn visit_ternaryop(&mut self, ternaryop: &ast::TernaryOp) {
        let ternaryop_node = self.new_node("TernOp:\n\'?:\'");

        let condition_node = self.counter;
        ternaryop.condition.accept(self);
        self.connect_nodes_with_label(condition_node, ternaryop_node, "cond");

        let true_node = self.counter;
        ternaryop.true_expr.accept(self);
        self.connect_nodes_with_label(true_node, ternaryop_node, "cond");

        let false_node = self.counter;
        ternaryop.false_expr.accept(self);
        self.connect_nodes_with_label(false_node, ternaryop_node, "cond");
    }

    fn visit_binaryop(&mut self, binaryop: &ast::BinaryOp) {
        let binaryop_node = self.new_node(&format!(
            "BinOp:\n{}",
            TOKEN_TYPE_STRING_MAP
                .get_by_left(&binaryop.token.ttype())
                .unwrap()
        ));

        let left_node = self.counter;
        binaryop.left.accept(self);
        self.connect_nodes(left_node, binaryop_node);

        let right_node = self.counter;
        binaryop.right.accept(self);
        self.connect_nodes(right_node, binaryop_node);
    }

    fn visit_unaryop(&mut self, unaryop: &ast::UnaryOp) {
        let unaryop_node = self.new_node(&format!(
            "UnOp:\n{}",
            TOKEN_TYPE_STRING_MAP
                .get_by_left(&unaryop.token.ttype())
                .unwrap()
        ));

        let operand_node = self.counter;
        unaryop.operand.accept(self);
        self.connect_nodes(operand_node, unaryop_node);
    }

    fn visit_group(&mut self, group: &ast::Group) {
        let group_node = self.new_node("Group\n\'(...)\'");

        for expression in group.expressions.iter() {
            let expression_node = self.counter;
            expression.accept(self);
            self.connect_nodes(expression_node, group_node);
        }
    }

    fn visit_function(&mut self, function: &ast::Function) {
        let function_node = self.new_node(&format!(
            "Function:\n${}(...)",
            function
                .start_token
                .value
                .as_ref()
                .expect("Token in Function must have value!")
        ));

        for (i, expression) in function.arguments.iter().enumerate() {
            let expression_node = self.counter;
            expression.accept(self);
            self.connect_nodes_with_label(
                expression_node,
                function_node,
                &format!("a{}", i + 1),
            );
        }
    }

    fn visit_integer(&mut self, integer: &ast::Integer) {
        self.new_node(&format!(
            "Int:\n{}",
            integer
                .token
                .value
                .as_ref()
                .expect("Token in Integer must have value!")
        ));
    }

    fn visit_stringnode(&mut self, stringnode: &ast::StringNode) {
        //TODO trim string
        self.new_node(&format!(
            "String:\n{}",
            stringnode
                .token
                .value
                .as_ref()
                .expect("Token in StringNode must have value!")
        ));
    }

    fn visit_substitution(&mut self, substitution: &ast::Substitution) {
        self.new_node(&format!(
            "Sub:\n{}",
            substitution
                .token
                .value
                .as_ref()
                .expect("Token in Substitution must have value!")
        ));
    }

    fn visit_driveletter(&mut self, driveletter: &ast::DriveLetter) {
        self.new_node(&format!(
            "Drive: {}:\\",
            driveletter
                .token
                .value
                .as_ref()
                .expect("Token in DriveLetter must have value!")
        ));
    }

    fn visit_tag(&mut self, tag: &ast::Tag) {
        self.new_node(&format!(
            "Tag:\n<{}>",
            tag.token
                .value
                .as_ref()
                .expect("Token in Tag must have value!")
        ));
    }
}
