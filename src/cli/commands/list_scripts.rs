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
        let script_paths = self.config.get_script_paths()?;
        let mut scripts = Vec::new();

        for path in script_paths {
            let input_text = std::fs::read_to_string(path)?;
            scripts.push(Script::new(&input_text)?);
        }

        for script in scripts {
            print!("{}", script.name());

            if let Some(description) = script.description() {
                print!(": {}", description)
            }

            println!();
        }

        Ok(())
    }
}
