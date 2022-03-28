use crate::cli::ui;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tfmt::Script;

pub(crate) struct Config {
    path: PathBuf,
}

impl Config {
    pub(crate) const HISTORY_NAME: &'static str = env!("CARGO_PKG_NAME");
    pub(crate) const PREVIEW_PREFIX: &'static str = "[P] ";
    pub(crate) const SCRIPT_EXTENSION: &'static str = "tfmt";

    pub(crate) fn new(path: &Path) -> Result<Self> {
        let config = Self {
            path: path.to_owned(),
        };

        Config::create_dir(&config.path)?;

        Ok(config)
    }

    pub(crate) fn default_path() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(format!(".{}", env!("CARGO_PKG_NAME")));

            Ok(path)
        } else {
            bail!("Unable to read home directory!")
        }
    }

    /// Search a path for files matching `predicate`, recursing for `depth`.
    pub(crate) fn search_path<P>(
        path: &Path,
        depth: usize,
        predicate: &P,
        spinner: Option<&ui::AudioFileSpinner>,
    ) -> Vec<PathBuf>
    where
        P: Fn(&Path) -> bool,
    {
        let mut found_paths = Vec::new();

        if let Ok(iter) = std::fs::read_dir(path) {
            for entry in iter.flatten() {
                let entry_path = entry.path();

                let matches_predicate = predicate(&entry_path);

                if entry_path.is_file() {
                    if let Some(spinner) = spinner {
                        spinner.inc_total();
                    }

                    if matches_predicate {
                        if let Some(spinner) = spinner {
                            spinner.inc_found();
                        }
                        found_paths.push(entry_path);
                    }
                } else if entry_path.is_dir() && depth > 0 {
                    found_paths.extend(Config::search_path(
                        &entry_path,
                        depth - 1,
                        predicate,
                        spinner,
                    ));
                }
            }
        }

        found_paths
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn get_scripts(&self) -> Result<Vec<Script>> {
        let paths = self.get_script_paths()?;

        let mut scripts = Vec::new();

        for path in paths {
            let input_text = std::fs::read_to_string(path)?;
            scripts.push(Script::new(&input_text)?);
        }

        Ok(scripts)
    }

    pub(crate) fn get_script(&self, name: &str) -> Result<Script> {
        let scripts = self.get_scripts()?;
        let found_scripts: Vec<Script> =
            scripts.into_iter().filter(|s| s.name() == name).collect();

        let length = found_scripts.len();

        if length == 0 {
            bail!("Unable to find script \"{}\"", name);
        } else if length > 1 {
            bail!("Found {} scripts with name \"{}\"", length, name);
        }

        let script = found_scripts.into_iter().next();

        // This unwrap is always safe, as we check the length manually.
        debug_assert!(script.is_some());

        Ok(script.unwrap())
    }

    fn create_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir(&path)?;
        } else if !path.is_dir() {
            bail!("Unable to create configuration directory!")
        }

        Ok(())
    }

    fn get_script_paths(&self) -> Result<Vec<PathBuf>> {
        let predicate: fn(&Path) -> bool = |p| {
            p.extension()
                .map_or(false, |s| s == Config::SCRIPT_EXTENSION)
        };

        let mut paths = Config::search_path(self.path(), 0, &predicate, None);
        paths.extend(Config::search_path(
            &std::env::current_dir()?,
            0,
            &predicate,
            None,
        ));

        Ok(paths)
    }
}
