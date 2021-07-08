use super::ast::{self, Expression, Node};
use super::visitor::Visitor;
use crate::error::TFMTError;
use crate::tfmt::token::Token;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

#[derive(Default)]
pub struct GenAstDot {
    counter: u64,
}

pub fn visualize_ast(
    root: ast::Program,
    directory: &Path,
    name: &str,
    remove_dot_file: bool,
) -> Result<()> {
    let mut g: GenAstDot = Default::default();

    let mut dot = "digraph astgraph {\n  \
        edge [arrowsize=.5];\n  \
        rankdir=\"TB\";\n  \
        newrank=true;\n  \
        nodesep=0.75;\n  \
        ranksep=0.75;\n  "
        .to_owned();

    dot.push_str(&root.accept(&mut g));

    dot.push('}');

    let mut path = PathBuf::from(directory);
    path.push(format!("{}.dot", name));

    fs::create_dir_all(directory)?;

    let mut file = fs::File::create(&path)?;
    file.write_all(dot.as_bytes())?;

    let spawn_result = Command::new("dot")
        .current_dir(directory)
        .arg("-Tpng")
        .args(&["-o", &format!("{}.png", name)])
        .arg(format!("{}.dot", name))
        .spawn();

    if let Ok(mut child) = spawn_result {
        child.wait()?
    } else {
        return Err(TFMTError::GenAstDot.into());
    };

    if remove_dot_file {
        fs::remove_file(path)?;
    }

    Ok(())
}

impl GenAstDot {
    fn increment(&mut self) -> u64 {
        self.counter += 1;
        self.counter - 1
    }

    fn create_node(&mut self, label: &str, hidden: bool) -> (String, u64) {
        let label_str = {
            if hidden {
                "[shape=point]".to_owned()
            } else {
                format!("[label=\"{}\"]", label.replace("\n", "\\n"))
            }
        };

        (
            format!("  node{} {}\n", self.counter, label_str),
            self.increment(),
        )
    }

    fn new_node(&mut self, label: &str) -> (String, u64) {
        self.create_node(label, false)
    }

    fn hidden_node(&mut self) -> (String, u64) {
        self.create_node("", true)
    }

    fn node_connector(
        &mut self,
        node1: u64,
        node2: u64,
        label: Option<&str>,
        directed: bool,
    ) -> String {
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

        string.push('\n');

        string
    }

    fn connect_nodes(&mut self, node1: u64, node2: u64) -> String {
        self.node_connector(node1, node2, None, true)
    }

    fn connect_nodes_with_label(
        &mut self,
        node1: u64,
        node2: u64,
        label: &str,
    ) -> String {
        self.node_connector(node1, node2, Some(label), true)
    }
}

impl Visitor<String> for GenAstDot {
    fn visit_program(&mut self, program: &ast::Program) -> String {
        let (mut string, program_node) = self.new_node(&format!(
            "Program\n{}",
            program
                .name
                .value
                .as_ref()
                .expect("Name required for program!")
        ));

        string += "subgraph header {\nrankdir=\"RL\";\n";

        let parameters_node = self.counter;

        string += &program.parameters.accept(self);

        string += &self.connect_nodes(parameters_node, program_node);

        string += "}\nsubgraph block  {\nrankdir=\"LR\";\n";

        let block_node = self.counter;
        string += &program.block.accept(self);
        string += &self.connect_nodes(block_node, program_node);

        string += "}\n";

        string
    }

    fn visit_parameters(&mut self, parameters: &ast::Parameters) -> String {
        let (mut string, parameters_node) = self
            .new_node(&format!("Params:\n({})", parameters.parameters.len()));

        for parameter in parameters.parameters.iter() {
            let parameter_node = self.counter;
            string += &parameter.accept(self);
            string += &self.connect_nodes(parameter_node, parameters_node);
        }

        string
    }

    fn visit_parameter(&mut self, parameter: &ast::Parameter) -> String {
        let (mut string, parameter_node) = self.new_node(
            parameter
                .token
                .value
                .as_ref()
                .expect("Parameter token must have value!"),
        );

        if let Some(default) = parameter.default.as_ref() {
            let (default_string, default_node) = self.new_node(
                default
                    .value
                    .as_ref()
                    .expect("Parameter default token must have value!"),
            );

            string += &default_string;
            string += &self.connect_nodes(default_node, parameter_node);
        }

        string
    }

    fn visit_block(&mut self, block: &ast::Block) -> String {
        let (mut string, block_node) = self.new_node("Block");

        if let Some(drive) = &block.drive {
            let drive_node = self.counter;
            string += &drive.accept(self);
            string += &self.connect_nodes_with_label(
                drive_node,
                block_node,
                "drive letter",
            );
        }

        let (expressions_string, expressions_node) = self.hidden_node();
        string += &expressions_string;
        string += &self.node_connector(
            expressions_node,
            block_node,
            Some("expressions"),
            false,
        );

        for expression in block.expressions.iter() {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &self.connect_nodes(expression_node, expressions_node)
        }

        string
    }

    fn visit_driveletter(&mut self, driveletter: &ast::DriveLetter) -> String {
        let (string, _) = self.new_node(&format!(
            "Drive: {}:\\",
            driveletter
                .token
                .value
                .as_ref()
                .expect("Token in DriveLetter must have value!")
        ));

        string
    }

    fn visit_expression(&mut self, expression: &Expression) -> String {
        match expression {
            Expression::TernaryOp {
                condition,
                true_expr,
                false_expr,
            } => self.visit_ternaryop(condition, true_expr, false_expr),
            Expression::BinaryOp { left, token, right } => {
                self.visit_binaryop(left, token, right)
            }
            Expression::UnaryOp { token, operand } => {
                self.visit_unaryop(token, operand)
            }
            Expression::Group { expressions } => self.visit_group(expressions),
            Expression::Function {
                start_token,
                arguments,
                ..
            } => self.visit_function(start_token, arguments),
            Expression::StringNode(string) => self.visit_string(string),
            Expression::IntegerNode(integer) => self.visit_integer(integer),
            Expression::Substitution(subst) => self.visit_substitution(subst),
            Expression::Tag { token, .. } => self.visit_tag(token),
        }
    }
}

impl GenAstDot {
    fn visit_ternaryop(
        &mut self,
        condition: &Expression,
        true_expr: &Expression,
        false_expr: &Expression,
    ) -> String {
        let (mut string, ternaryop_node) = self.new_node("TernOp:\n\'?:\'");

        let condition_node = self.counter;
        string += &condition.accept(self);
        string += &self.connect_nodes_with_label(
            condition_node,
            ternaryop_node,
            "cond",
        );

        let true_node = self.counter;
        string += &true_expr.accept(self);
        string +=
            &self.connect_nodes_with_label(true_node, ternaryop_node, "cond");

        let false_node = self.counter;
        string += &false_expr.accept(self);
        string +=
            &self.connect_nodes_with_label(false_node, ternaryop_node, "cond");

        string
    }

    fn visit_binaryop(
        &mut self,
        left: &Expression,
        token: &Token,
        right: &Expression,
    ) -> String {
        let (mut string, binaryop_node) =
            self.new_node(&format!("BinOp:\n{}", token.ttype.grapheme()));

        let left_node = self.counter;
        string += &left.accept(self);
        string += &self.connect_nodes(left_node, binaryop_node);

        let right_node = self.counter;
        string += &right.accept(self);
        string += &self.connect_nodes(right_node, binaryop_node);

        string
    }

    fn visit_unaryop(&mut self, token: &Token, operand: &Expression) -> String {
        let (mut string, unaryop_node) =
            self.new_node(&format!("UnOp:\n{}", token.ttype.grapheme()));

        let operand_node = self.counter;
        string += &operand.accept(self);
        string += &self.connect_nodes(operand_node, unaryop_node);

        string
    }

    fn visit_group(&mut self, expressions: &[Expression]) -> String {
        let (mut string, group_node) = self.new_node("Group\n\'(...)\'");

        for expression in expressions.iter() {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &self.connect_nodes(expression_node, group_node);
        }

        string
    }

    fn visit_function(
        &mut self,
        start_token: &Token,
        arguments: &[Expression],
    ) -> String {
        let (mut string, function_node) = self.new_node(&format!(
            "Function:\n${}(...)",
            start_token
                .value
                .as_ref()
                .expect("Token in Function must have value!")
        ));

        for (i, expression) in arguments.iter().enumerate() {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &self.connect_nodes_with_label(
                expression_node,
                function_node,
                &format!("a{}", i + 1),
            );
        }

        string
    }

    fn visit_integer(&mut self, integer: &Token) -> String {
        let (string, _) = self.new_node(&format!(
            "Int:\n{}",
            integer
                .value
                .as_ref()
                .expect("Token in Integer must have value!")
        ));

        string
    }

    fn visit_string(&mut self, string: &Token) -> String {
        // TODO trim string
        let (string, _) = self.new_node(&format!(
            "String:\n{}",
            string
                .value
                .as_ref()
                .expect("Token in StringNode must have value!")
        ));

        string
    }

    fn visit_substitution(&mut self, substitution: &Token) -> String {
        let (string, _) = self.new_node(&format!(
            "Sub:\n{}",
            substitution
                .value
                .as_ref()
                .expect("Token in Substitution must have value!")
        ));

        string
    }

    fn visit_tag(&mut self, token: &Token) -> String {
        let (string, _) = self.new_node(&format!(
            "Tag:\n<{}>",
            token.value.as_ref().expect("Token in Tag must have value!")
        ));

        string
    }
}
