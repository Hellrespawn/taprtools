use crate::cli::Config;
use anyhow::Result;

pub(crate) struct InspectScript {
    config: Config,
}

impl InspectScript {
    pub(crate) fn new(config: Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self, name: &str) -> Result<()> {
        todo!()
    }
}
