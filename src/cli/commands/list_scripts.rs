use crate::cli::Config;
use anyhow::Result;

pub(crate) struct ListScripts {
    config: Config,
}

impl ListScripts {
    pub(crate) fn new(config: Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self) -> Result<()> {
        todo!()
    }
}
