use anyhow::{bail, Result};
use tempfile::{Builder, TempDir};

fn setup_environment(suffix: Option<&str>) -> Result<TempDir> {
    let t = if let Some(suffix) = suffix {
        Builder::new()
            .prefix("file-history-")
            .suffix(&("-".to_string() + suffix))
            .tempdir()?
    } else {
        Builder::new().prefix("file-history-").tempdir()?
    };

    Ok(t)
}

// TODO Write tests for file-history
