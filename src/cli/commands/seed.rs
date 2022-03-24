use crate::cli::Config;
use anyhow::{bail, Result};

struct DefaultFile {
    name: &'static str,
    content: &'static str,
}

static DEFAULT_FILES: [DefaultFile; 1] = [DefaultFile {
    name: "sync.tfmt",
    content: include_str!("../../../doc/sync.tfmt"),
}];

pub(crate) fn seed(config: &Config) -> Result<()> {
    let existing_files: Vec<&DefaultFile> = DEFAULT_FILES
        .iter()
        .filter(|file| config.path().join(file.name).exists())
        .collect();

    if !existing_files.is_empty() {
        bail!(
            "The following files already exist:\n{}",
            existing_files
                .iter()
                .map(|f| f.name)
                .collect::<Vec<&str>>()
                .join("\n")
        );
    }

    for file in &DEFAULT_FILES {
        let path = config.path().join(file.name);

        std::fs::write(path, file.content)?;
        println!("Wrote default files to {}", config.path().display());
    }

    Ok(())
}
