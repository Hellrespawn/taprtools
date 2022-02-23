use crate::cli::Config;
use anyhow::Result;
use tfmt::Script;

pub(crate) fn list_scripts(config: &Config) -> Result<()> {
    let scripts = config.get_scripts()?;

    if scripts.is_empty() {
        println!("Couldn't find any scripts.");
    } else {
        println!("Scripts:");
    }

    for script in scripts {
        print_script_info(&script);
    }

    Ok(())
}

fn print_script_info(script: &Script) {
    print!("{}", script.name());

    if let Some(description) = script.description() {
        print!(": {}", description);
    }

    println!();
}
