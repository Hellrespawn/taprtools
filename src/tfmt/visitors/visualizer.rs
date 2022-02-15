use crate::tfmt::ast::node::{self, Expression, Node};
use crate::tfmt::ast::Visitor;
use crate::tfmt::error::DotError;
use crate::tfmt::token::Token;
use anyhow::Result;
use log::{info, trace};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[allow(clippy::doc_markdown)]
/// A [`Visitor`] used to construct a GraphViz dot-file.
pub struct Visualizer {
    counter: u64,
}

impl Visualizer {
    #[allow(clippy::doc_markdown)]
    /// Construct a GraphViz dot-file from a [`node::Program`] and render it as a png.
    pub fn visualize_ast<P: AsRef<Path>>(
        program: &node::Program,
        directory: &P,
        name: &str,
        remove_dot_file: bool,
    ) -> Result<()> {
        let mut g = Self { counter: 0 };

        let mut dot = "digraph astgraph {\n  \
            edge [arrowsize=.5];\n  \
            rankdir=\"TB\";\n  \
            newrank=true;\n  \
            nodesep=0.75;\n  \
            ranksep=0.75;\n  "
            .to_owned();

        dot.push_str(&program.accept(&mut g));

        dot.push('}');

        trace!("Generated .dot-file:\n{}", dot);

        let path =
            PathBuf::from(directory.as_ref()).join(format!("{}.dot", name));

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
            return Err(DotError::CantRun.into());
        };

        if remove_dot_file {
            fs::remove_file(&path)?;
        }

        info!(
            "Rendered Abstract Syntax Tree to {}",
            &path.with_extension("png").display()
        );

        Ok(())
    }

    fn increment(&mut self) -> u64 {
        self.counter += 1;
        self.counter - 1
    }

    fn create_node(&mut self, label: &str, hidden: bool) -> (String, u64) {
        let label_str = {
            if hidden {
                "[shape=point]".to_owned()
            } else {
                format!(r#"[label="{}"]"#, label.replace('\n', "\\n"))
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
        node1: u64,
        node2: u64,
        label: Option<&str>,
        directed: bool,
    ) -> String {
        let mut string = format!("  node{} -> node{}", node2, node1);

        let mut args: Vec<String> = Vec::new();

        if let Some(label) = label {
            args.push(format!(r#"label="{}""#, label.replace('\n', "\\n")));
        }

        if !directed {
            args.push("dir=none".to_owned());
        }

        if !args.is_empty() {
            string.push_str(&format!(" [{}]", args.join(", ")));
        }

        string.push('\n');

        string
    }

    fn connect_nodes(node1: u64, node2: u64) -> String {
        Visualizer::node_connector(node1, node2, None, true)
    }

    fn connect_nodes_with_label(node1: u64, node2: u64, label: &str) -> String {
        Visualizer::node_connector(node1, node2, Some(label), true)
    }
}

impl Visitor<String> for Visualizer {
    fn visit_program(&mut self, program: &node::Program) -> String {
        let (mut string, program_node) = self.new_node(&format!(
            "Program\n{}",
            program.name.get_string_unchecked()
        ));

        string += "subgraph header {\nrankdir=\"RL\";\n";

        let parameters_node = self.counter;

        string += &program.parameters.accept(self);

        string += &Visualizer::connect_nodes(parameters_node, program_node);

        string += "}\nsubgraph block  {\nrankdir=\"LR\";\n";

        let block_node = self.counter;
        string += &program.block.accept(self);
        string += &Visualizer::connect_nodes(block_node, program_node);

        string += "}\n";

        string
    }

    // "Parameters" and "Parameter" are simply that similar
    #[allow(clippy::similar_names)]
    fn visit_parameters(&mut self, parameters: &node::Parameters) -> String {
        let (mut string, parameters_node) = self
            .new_node(&format!("Params:\n({})", parameters.parameters.len()));

        for parameter in &parameters.parameters {
            let parameter_node = self.counter;
            string += &parameter.accept(self);
            string +=
                &Visualizer::connect_nodes(parameter_node, parameters_node);
        }

        string
    }

    fn visit_parameter(&mut self, parameter: &node::Parameter) -> String {
        let (mut string, parameter_node) =
            self.new_node(parameter.token.get_string_unchecked());

        if let Some(default) = parameter.default.as_ref() {
            let (default_string, default_node) =
                self.new_node(default.get_string_unchecked());

            string += &default_string;
            string += &Visualizer::connect_nodes(default_node, parameter_node);
        }

        string
    }

    fn visit_block(&mut self, block: &node::Block) -> String {
        let (mut string, block_node) = self.new_node("Block");

        let (hidden_node_string, hidden_node) = self.hidden_node();
        string += &hidden_node_string;
        string += &Visualizer::node_connector(
            hidden_node,
            block_node,
            Some("expressions"),
            false,
        );

        for expression in &block.expressions {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &Visualizer::connect_nodes(expression_node, hidden_node);
        }

        string
    }

    fn visit_ternaryop(
        &mut self,
        condition: &Expression,
        true_expr: &Expression,
        false_expr: &Expression,
    ) -> String {
        let (mut string, ternaryop_node) = self.new_node("TernOp:\n\'?:\'");

        let condition_node = self.counter;
        string += &condition.accept(self);
        string += &Visualizer::connect_nodes_with_label(
            condition_node,
            ternaryop_node,
            "cond",
        );

        let true_node = self.counter;
        string += &true_expr.accept(self);
        string += &Visualizer::connect_nodes_with_label(
            true_node,
            ternaryop_node,
            "cond",
        );

        let false_node = self.counter;
        string += &false_expr.accept(self);
        string += &Visualizer::connect_nodes_with_label(
            false_node,
            ternaryop_node,
            "cond",
        );

        string
    }

    fn visit_binaryop(
        &mut self,
        left: &Expression,
        token: &Token,
        right: &Expression,
    ) -> String {
        let (mut string, binaryop_node) =
            self.new_node(&format!("BinOp:\n{:?}", token.token_type));

        let left_node = self.counter;
        string += &left.accept(self);
        string += &Visualizer::connect_nodes(left_node, binaryop_node);

        let right_node = self.counter;
        string += &right.accept(self);
        string += &Visualizer::connect_nodes(right_node, binaryop_node);

        string
    }

    fn visit_unaryop(&mut self, token: &Token, operand: &Expression) -> String {
        let (mut string, unaryop_node) =
            self.new_node(&format!("UnOp:\n{:?}", token.token_type));

        let operand_node = self.counter;
        string += &operand.accept(self);
        string += &Visualizer::connect_nodes(operand_node, unaryop_node);

        string
    }

    fn visit_group(&mut self, expressions: &[Expression]) -> String {
        let (mut string, group_node) = self.new_node("Group\n\'(...)\'");

        for expression in expressions.iter() {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &Visualizer::connect_nodes(expression_node, group_node);
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
            start_token.get_string_unchecked()
        ));

        for (i, expression) in arguments.iter().enumerate() {
            let expression_node = self.counter;
            string += &expression.accept(self);
            string += &Visualizer::connect_nodes_with_label(
                expression_node,
                function_node,
                &format!("a{}", i + 1),
            );
        }

        string
    }

    fn visit_integer(&mut self, integer: &Token) -> String {
        let (string, _) =
            self.new_node(&format!("Int:\n{}", integer.get_int_unchecked()));

        string
    }

    fn visit_string(&mut self, string: &Token) -> String {
        let (string, _) = self.new_node(&format!(
            "String:\n{}",
            string.get_string_unchecked().trim()
        ));

        string
    }

    fn visit_symbol(&mut self, symbol: &Token) -> String {
        let (string, _) =
            self.new_node(&format!("Sym:\n{}", symbol.get_string_unchecked()));

        string
    }

    fn visit_tag(&mut self, token: &Token) -> String {
        let (string, _) =
            self.new_node(&format!("Tag:\n<{}>", token.get_string_unchecked()));

        string
    }
}
