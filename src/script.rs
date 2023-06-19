use anyhow::{bail, Result};
use std::path::Path;
use tapr::{Environment, Interpreter, Node, NodeData, Value, Visitor};

/// Reads a script, parses an AST and gets the name, description and parameters.
#[derive(Debug)]
pub struct Script {
    name: String,
    parameters: Vec<String>,
    node: Node,
}

impl Script {
    /// Create a new Script instance.
    pub fn from_file(path: &Path) -> Result<Self> {
        let name = path
            .file_stem()
            .map(std::ffi::OsStr::to_string_lossy)
            .expect("File should have a file name.");

        let body = std::fs::read_to_string(path)?;

        let node = Node::from_string(&body, &name)?;

        let mut intp = Interpreter::default();
        intp.push_environment(Environment::new());

        node.accept(&mut intp)?;

        let env: Environment = intp.pop_environment();

        let Some(value) = env.get("rename") else {
            bail!("Script did not include rename function.")
        };

        let Value::Callable(callable) = value else {
            bail!("Script did not include rename function.")
        };

        let parameters = callable
            .parameters()
            .iter()
            .map(|p| p.name().to_owned())
            .collect();

        Ok(Script {
            name: name.to_string(),
            parameters,
            node,
        })
    }

    /// Returns the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parameters
    pub fn parameters(&self) -> &[String] {
        &self.parameters
    }

    /// Accepts a visitor
    pub(crate) fn accept<T: std::fmt::Debug>(
        &self,
        visitor: &mut dyn Visitor<T>,
    ) -> T {
        self.node.accept(visitor)
    }

    pub fn add_arguments_to_node(
        &mut self,
        arguments: &[String],
    ) -> Result<()> {
        if self.parameters.len() != arguments.len() {
            bail!(
                "Script expected {} args, received {}",
                self.parameters.len(),
                arguments.len()
            )
        }

        let NodeData::Main(main_nodes) = self.node.data_mut() else {
            panic!("Root node of script '{}' is not Node::main", self.name);
        };

        main_nodes.push(Self::create_mock_node(arguments));

        Ok(())
    }

    fn create_mock_node(arguments: &[String]) -> Node {
        let mut nodes = vec![Node::mock(NodeData::Symbol {
            module: None,
            value: "rename".to_owned(),
        })];

        nodes.extend(
            arguments
                .iter()
                .map(|s| Node::mock(NodeData::String(s.clone()))),
        );

        Node::mock(NodeData::List {
            literal: false,
            nodes,
        })
    }
}
