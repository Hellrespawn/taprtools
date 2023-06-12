#![cfg(feature = "graphviz")]
use crate::cli::Config;
use anyhow::{anyhow, Result};
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) fn render_script(config: &Config, name: &str) -> Result<()> {
    let script = config.get_script(name)?;

    let dot = script.create_ast_dot();
    render_ast(&dot, script.name())?;

    Ok(())
}
fn render_ast(dot: &str, name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let name = format!("{}-ast.png", name);

    let spawn_result = Command::new("dot")
        .stdin(Stdio::piped())
        .current_dir(cwd)
        .arg("-Tpng")
        .args(["-o", &name])
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
