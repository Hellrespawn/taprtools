use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};
use tfmt::Script;

use crate::cli::Filesystem;

pub(crate) struct InspectScript;

impl InspectScript {
    pub(crate) fn run(name: &str, render_ast: bool) -> Result<()> {
        let script = Filesystem::get_script(name)?;

        InspectScript::print_script_info(&script);

        if render_ast {
            let dot = script.create_ast_dot();
            InspectScript::render_ast(&dot, script.name())?;
        }

        Ok(())
    }

    fn print_script_info(script: &Script) {
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

    fn render_ast(dot: &str, name: &str) -> Result<()> {
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
