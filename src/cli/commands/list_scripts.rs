use crate::cli::Config;
use crate::script::Script;
use anyhow::Result;

pub(crate) fn list_scripts(config: &Config) -> Result<()> {
    let scripts = config.get_scripts()?;

    if scripts.is_empty() {
        println!(
            "Couldn't find any scripts at {} or in the current directory.",
            config.path().display()
        );
    } else {
        println!("Scripts:");
    }

    for script in scripts {
        print_script_info(&script);
    }

    Ok(())
}

fn print_script_info(script: &Script) {
    print!("{}[{}]", script.name(), script.parameters().join(" "));
}
