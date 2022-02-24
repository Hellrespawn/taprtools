use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tfmt::Script;

use crate::file::AudioFile;

pub(crate) struct Config {
    path: PathBuf,
}

impl Config {
    pub(crate) const HISTORY_FILENAME: &'static str = "tfmttools.hist";
    pub(crate) const PREVIEW_PREFIX: &'static str = "[P] ";
    pub(crate) const SCRIPT_EXTENSION: &'static str = "tfmt";

    pub(crate) fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let config = Self {
            path: path.as_ref().to_path_buf(),
        };

        Config::create_dir(&config.path)?;

        Ok(config)
    }

    pub(crate) fn default() -> Result<Self> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(format!(".{}", env!("CARGO_PKG_NAME")));

            let config = Self { path };

            Config::create_dir(&config.path)?;

            Ok(config)
        } else {
            bail!("Unable to read home directory!")
        }
    }

    /// Search a path for files matching `predicate`, recursing for `depth`.
    fn search_path<P, Q>(path: &P, depth: usize, predicate: &Q) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
        Q: Fn(&Path) -> bool,
    {
        let mut found_paths = Vec::new();

        if let Ok(iter) = std::fs::read_dir(path) {
            for entry in iter.flatten() {
                let entry_path = entry.path();

                let matches_predicate = predicate(&entry_path);

                if entry_path.is_file() && matches_predicate {
                    found_paths.push(entry_path);
                } else if entry_path.is_dir() && depth > 0 {
                    found_paths.extend(Config::search_path(
                        &entry_path,
                        depth - 1,
                        predicate,
                    ));
                }
            }
        }

        found_paths
    }

    fn create_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir(&path)?;
        } else if !path.is_dir() {
            bail!("Unable to create project directory!")
        }

        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn get_history_path(&self) -> PathBuf {
        let path = self.path().join(Config::HISTORY_FILENAME);
        path
    }

    fn get_script_paths(&self) -> Result<Vec<PathBuf>> {
        let closure: fn(&Path) -> bool = |p| {
            p.extension()
                .map_or(false, |s| s == Config::SCRIPT_EXTENSION)
        };

        let mut paths = Config::search_path(&self.path(), 0, &closure);
        paths.extend(Config::search_path(
            &std::env::current_dir()?,
            0,
            &closure,
        ));

        Ok(paths)
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
            bail!("Unable to find script \"{name}\"");
        } else if length > 1 {
            bail!("Found {length} scripts with name \"{name}\"");
        }

        let script = found_scripts.into_iter().next();

        // This unwrap is always safe, as we check the length manually.
        debug_assert!(script.is_some());

        Ok(script.unwrap())
    }

    pub(crate) fn get_audiofiles(
        recursion_depth: usize,
    ) -> Result<Vec<AudioFile>> {
        let path = std::env::current_dir()?;

        let paths = Config::search_path(&path, recursion_depth, &|p| {
            p.extension().map_or(false, |extension| {
                for supported_extension in AudioFile::SUPPORTED_EXTENSIONS {
                    if extension == supported_extension {
                        return true;
                    }
                }

                false
            })
        });

        paths.iter().map(AudioFile::new).collect()
    }
}
