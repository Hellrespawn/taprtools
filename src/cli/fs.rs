use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tfmt::Script;

use crate::file::AudioFile;

pub(crate) struct Filesystem;

impl Filesystem {
    pub(crate) const HISTORY_FILENAME: &'static str = "tfmttools.hist";
    pub(crate) const PREVIEW_PREFIX: &'static str = "[P] ";
    pub(crate) const SCRIPT_EXTENSION: &'static str = "tfmt";

    /// Search a path for files matching `predicate`, recursing for `depth`.
    fn search_path<P, Q>(path: &P, depth: usize, predicate: Q) -> Vec<PathBuf>
    where
        P: AsRef<Path>,
        // TODO Find out why Copy is necessary.
        Q: Copy + Fn(&Path) -> bool,
    {
        let mut found_paths = Vec::new();

        if let Ok(iter) = std::fs::read_dir(path) {
            for entry in iter.flatten() {
                let entry_path = entry.path();

                if entry_path.is_file() && predicate(&entry_path) {
                    found_paths.push(entry_path);
                } else if entry_path.is_dir() && depth > 0 {
                    found_paths.extend(Filesystem::search_path(
                        &entry_path,
                        depth - 1,
                        predicate,
                    ));
                }
            }
        }

        found_paths
    }

    fn get_project_dir() -> Result<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            let project_dir = home.join(format!(".{}", env!("CARGO_PKG_NAME")));
            if !project_dir.exists() {
                std::fs::create_dir(&project_dir)?;
            } else if !project_dir.is_dir() {
                bail!("Unable to create project directory!")
            }

            Ok(project_dir)
        } else {
            bail!("Unable to read home directory!")
        }
    }

    pub(crate) fn get_history_path() -> Result<PathBuf> {
        let path =
            Filesystem::get_project_dir()?.join(Filesystem::HISTORY_FILENAME);
        Ok(path)
    }

    fn get_script_paths() -> Result<Vec<PathBuf>> {
        // FIXME also get scripts in cwd?
        let path = Filesystem::get_project_dir()?;

        let paths = Filesystem::search_path(&path, 0, |p| {
            p.extension()
                .map_or(false, |s| s == Filesystem::SCRIPT_EXTENSION)
        });

        Ok(paths)
    }

    pub(crate) fn get_scripts() -> Result<Vec<Script>> {
        let paths = Filesystem::get_script_paths()?;

        let mut scripts = Vec::new();

        for path in paths {
            let input_text = std::fs::read_to_string(path)?;
            scripts.push(Script::new(&input_text)?);
        }

        Ok(scripts)
    }

    pub(crate) fn get_script(name: &str) -> Result<Script> {
        let scripts = Filesystem::get_scripts()?;
        let found: Vec<Script> =
            scripts.into_iter().filter(|s| s.name() == name).collect();

        let length = found.len();

        if length == 0 {
            bail!("Unable to find script \"{name}\"");
        } else if length > 1 {
            bail!("Found {length} scripts with name \"{name}\"");
        }
        // This unwrap is always safe, as we check the length manually.
        Ok(found.into_iter().next().unwrap())
    }

    pub(crate) fn get_audiofiles(
        recursion_depth: usize,
    ) -> Result<Vec<AudioFile>> {
        let path = std::env::current_dir()?;

        let paths = Filesystem::search_path(&path, recursion_depth, |p| {
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
