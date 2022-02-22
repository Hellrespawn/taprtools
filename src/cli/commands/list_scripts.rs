use crate::cli::Filesystem;
use anyhow::Result;
use tfmt::Script;

pub(crate) struct ListScripts;

impl ListScripts {
    pub(crate) fn run() -> Result<()> {
        let scripts = Filesystem::get_scripts()?;

        if scripts.is_empty() {
            println!("Couldn't find any scripts.");
        } else {
            println!("Scripts:");
        }

        for script in scripts {
            Self::print_script_info(&script);
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
}
