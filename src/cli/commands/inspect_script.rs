use std::io::Write;
use std::process::{Command, Stdio};

use crate::cli::Config;
use anyhow::{anyhow, Result};
use tfmt::Script;

pub(crate) struct InspectScript<'a> {
    config: &'a Config,
}

impl<'a> InspectScript<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self, name: &str, render_ast: bool) -> Result<()> {
        let script = self.config.get_script(name)?;

        self.print_script_info(&script);

        if render_ast {
            let dot = script.create_ast_dot();
            self.render_ast(dot, script.name())?;
        }

        Ok(())
    }

    fn print_script_info(&self, script: &Script) {
        print!("{}(", script.name());

        let parameters = script.parameters();

        for (i, param) in parameters.iter().enumerate() {
            print!("{}", param.name());

            if let Some(default) = param.default() {
                print!("={}", default);
            }

            if i < parameters.len() - 1 {
                print!(", ");
            }
        }

        println!("):");

        if let Some(description) = script.description() {
            println!("{}", description);
        }

        println!();
    }

    fn render_ast(&self, dot: String, name: &str) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let name = format!("{}-ast.png", name);

        let spawn_result = Command::new("dot")
            .stdin(Stdio::piped())
            .current_dir(cwd)
            .arg("-Tpng")
            .args(&["-o", &name])
            .spawn();

        if let Ok(mut child) = spawn_result {
            child
                .stdin
                .as_ref()
                .ok_or_else(|| anyhow!("Unable to get stdin on subprocess!"))?
                .write_all(dot.as_bytes())?;

            child.wait()?;

            println!("Rendered AST to {}", &name);

            Ok(())
        } else {
            Err(anyhow!(
                "Unable to run dot! Is GraphViz installed and is it in PATH?"
            ))
        }
    }
}
