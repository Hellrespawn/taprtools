use crate::cli::Config;
use anyhow::Result;
use tfmt::Script;

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

    print!("): ");

    if let Some(description) = script.description() {
        println!("{}", description);
    }
}
