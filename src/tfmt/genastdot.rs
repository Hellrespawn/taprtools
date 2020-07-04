use super::ast::{self, Node};
use super::token::TOKEN_TYPE_STRING_MAP;

use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GenAstDot {
    node_count: u32,
    dot_body: Vec<String>,
}

pub fn visualize_ast(
    root: ast::Program,
    directory: &Path,
    name: &str,
    remove_dot_file: bool,
) -> Result<(), Box<dyn Error>> {
    let mut g = GenAstDot::new();

    root.accept(&mut g);

    let mut dot = "digraph astgraph {\n  \
        edge [arrowsize=.5];\n  \
        rankdir=\"TB\";\n  \
        newrank=true;\n  \
        nodesep=0.75;\n  \
        ranksep=0.75;\n  "
        .to_owned();

    dot.push_str(&g.dot_body.join(""));

    dot.push_str("}");

    let mut path = PathBuf::from(directory);
    path.push(format!("{}.dot", name));

    fs::create_dir_all(directory)?;

    let mut file = fs::File::create(&path)?;
    file.write_all(dot.as_bytes())?;

    let output = Command::new("dot")
        .current_dir(directory)
        .arg("-Tpng")
        .arg(format!("-o {:?}.png", name))
        .arg(format!("{:?}.dot", name))
        .output()
        .expect("Something failed!");

    println!("{:?}", output.stdout);

    if remove_dot_file {
        fs::remove_file(path)?;
    }

    Ok(())
}

impl GenAstDot {
    pub fn new() -> GenAstDot {
        GenAstDot {
            node_count: 0,
            dot_body: Vec::new(),
        }
    }

    fn create_node(&mut self, label: &str, hidden: bool) -> u32 {
        let label_str = {
            match hidden {
                true => "[shape=point]".to_owned(),
                false => format!("[label=\"{}\"]", label.replace("\n", "\\n")),
            }
        };

        self.dot_body
            .push(format!("node{} {}\n", self.node_count, label_str));

        self.node_count += 1;

        self.node_count - 1
    }

    fn new_node(&mut self, label: &str) -> u32 {
        self.create_node(label, false)
    }

    fn hidden_node(&mut self) -> u32 {
        self.create_node("", true)
    }

    fn node_connector(
        &mut self,
        node1: u32,
        node2: u32,
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

        self.dot_body.push(string);
    }

    fn connect_nodes(&mut self, node1: u32, node2: u32) {
        self.node_connector(node1, node2, None, true)
    }

    fn connect_nodes_with_label(
        &mut self,
        node1: u32,
        node2: u32,
        label: &str,
    ) {
        self.node_connector(node1, node2, Some(label), true)
    }
}

impl ast::Visitor for GenAstDot {
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
            .push("subgraph header {\nrankdir=\"RL\";\n".to_owned());

        let parameters_node = self.node_count;
        program.parameters.accept(self);
        self.connect_nodes(parameters_node, program_node);

        self.dot_body
            .push("}\nsubgraph block  {\nrankdir=\"LR\";\n".to_owned());

        let block_node = self.node_count;
        program.block.accept(self);
        self.connect_nodes(block_node, program_node);

        self.dot_body.push("}\n".to_owned());
    }

    fn visit_parameters(&mut self, parameters: &ast::Parameters) {
        let parameters_node = self.new_node(&format!(
            "Params:\n({})",
            parameters.parameters.len()
        ));

        for parameter in parameters.parameters.iter() {
            let parameter_node = self.node_count;
            parameter.accept(self);
            self.connect_nodes(parameter_node, parameters_node);
        }
    }

    fn visit_parameter(&mut self, parameter: &ast::Parameter) {
        let parameter_node = self.new_node(&format!(
            "{}",
            parameter
                .token
                .value
                .as_ref()
                .expect("Parameter token must have value!")
        ));

        if let Some(default) = parameter.default.as_ref() {
            let default_node = self.new_node(&format!(
                "{}",
                default
                    .value
                    .as_ref()
                    .expect("Parameter default token must have value!")
            ));
            self.connect_nodes(default_node, parameter_node);
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        let block_node = self.new_node("Block");

        if let Some(drive) = &block.drive {
            let drive_node = self.node_count;
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
            let expression_node = self.node_count;
            expression.accept(self);
            self.connect_nodes(expression_node, expressions_node)
        }
    }

    fn visit_ternaryop(&mut self, ternaryop: &ast::TernaryOp) {
        let ternaryop_node = self.new_node(&format!("TernOp:\n\'?:\'"));

        let condition_node = self.node_count;
        ternaryop.condition.accept(self);
        self.connect_nodes_with_label(condition_node, ternaryop_node, "cond");

        let true_node = self.node_count;
        ternaryop.true_expr.accept(self);
        self.connect_nodes_with_label(true_node, ternaryop_node, "cond");

        let false_node = self.node_count;
        ternaryop.false_expr.accept(self);
        self.connect_nodes_with_label(false_node, ternaryop_node, "cond");
    }

    fn visit_binaryop(&mut self, binaryop: &ast::BinaryOp) {
        let binaryop_node = self.new_node(&format!(
            "BinOp:\n{}",
            TOKEN_TYPE_STRING_MAP.get_by_left(&binaryop.token.ttype()).unwrap()
        ));

        let left_node = self.node_count;
        binaryop.left.accept(self);
        self.connect_nodes(left_node, binaryop_node);

        let right_node = self.node_count;
        binaryop.right.accept(self);
        self.connect_nodes(right_node, binaryop_node);
    }

    fn visit_unaryop(&mut self, unaryop: &ast::UnaryOp) {
        let unaryop_node = self.new_node(&format!(
            "UnOp:\n{}",
            TOKEN_TYPE_STRING_MAP.get_by_left(&unaryop.token.ttype()).unwrap()
        ));

        let operand_node = self.node_count;
        unaryop.operand.accept(self);
        self.connect_nodes(operand_node, unaryop_node);
    }

    fn visit_group(&mut self, group: &ast::Group) {
        let group_node = self.new_node(&format!("Group\n\'(...)\'"));

        for expression in group.expressions.iter() {
            let expression_node = self.node_count;
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
            let expression_node = self.node_count;
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
            tag
                .token
                .value
                .as_ref()
                .expect("Token in Tag must have value!")
        ));
    }
}
