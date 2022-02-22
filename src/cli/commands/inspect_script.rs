use crate::cli::Config;
use anyhow::Result;

pub(crate) struct InspectScript<'a> {
    config: &'a Config,
}

impl<'a> InspectScript<'a> {
    pub(crate) fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub(crate) fn run(&self, name: &str, render_ast: bool) -> Result<()> {
        todo!()
    }
}
