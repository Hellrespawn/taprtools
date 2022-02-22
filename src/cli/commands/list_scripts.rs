use crate::cli::Config;
use anyhow::Result;
use tfmt::Script;

pub(crate) struct ListScripts<'a> {
    config: &'a Config,
}

impl<'a> ListScripts<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self) -> Result<()> {
        let scripts = self.config.get_scripts()?;

        if scripts.is_empty() {
            println!("Couldn't find any scripts.");
        } else {
            println!("Scripts:");
        }

        for script in scripts {
            self.print_script_info(&script);
        }

        Ok(())
    }

    fn print_script_info(&self, script: &Script) {
        print!("{}", script.name());

        if let Some(description) = script.description() {
            print!(": {}", description);
        }

        println!();
    }
}
