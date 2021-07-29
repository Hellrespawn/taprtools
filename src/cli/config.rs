use anyhow::{anyhow, Result};
use log::debug;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

pub fn get_log_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("tfmttools");
    path
}
pub fn get_config_dirs() -> &'static [PathBuf] {
    static DIRS: Lazy<Vec<PathBuf>> = Lazy::new(|| {
        let config = match dirs::config_dir() {
            Some(dir) => {
                let mut dir = dir;
                dir.push("tfmttools");
                Some(dir)
            }
            None => None,
        };

        let home = match dirs::home_dir() {
            Some(dir) => {
                let mut dir = dir;
                dir.push(".tfmttools");
                Some(dir)
            }
            None => None,
        };

        let cwd = std::env::current_dir().ok();

        // testdata is added only when run from Cargo.
        let dirs = match std::env::var("CARGO_HOME") {
            Ok(_) => vec![Some(PathBuf::from("testdata")), home, config, cwd],
            Err(_) => vec![home, config, cwd],
        }
        .into_iter()
        .flatten()
        .filter(|p| p.is_dir())
        .collect();

        debug!("Valid config dirs:\n{:#?}", dirs);
        dirs
    });

    &DIRS
}

fn search_dir_for_extension<P: AsRef<Path>>(dir: &P, extension: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(found_ext) = path.extension() {
                        // TODO? Mime type or something?
                        if found_ext == extension {
                            paths.push(path);
                        }
                    }
                    // TODO? Do we want recursion?
                    // } else if file_type.is_dir() {
                    //     paths.extend(search_dir_for_extension(&path))
                }
            }
        }
    }

    paths
}

pub fn search_dir_for_filename<P: AsRef<Path>>(dir: &P, filename: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(found_fn) = path.file_name() {
                        // TODO? Mime type or something?
                        if found_fn == filename {
                            paths.push(path);
                        }
                    }
                    // TODO? Do we want recursion?
                    // } else if file_type.is_dir() {
                    //     paths.extend(search_dir_for_filename(&path))
                }
            }
        }
    }

    paths
}

pub fn get_all_scripts() -> Vec<PathBuf> {
    let mut scripts = Vec::new();

    for dir in get_config_dirs() {
        let mut dir = PathBuf::from(dir);
        scripts.extend(search_dir_for_extension(&dir, "tfmt"));

        dir.push("script");
        scripts.extend(search_dir_for_extension(&dir, "tfmt"));
        dir.pop();

        dir.push("scripts");
        scripts.extend(search_dir_for_extension(&dir, "tfmt"));
    }

    debug!("Found scripts:\n{:#?}", scripts);

    scripts
}

pub fn get_script(name: &str) -> Result<PathBuf> {
    let name = format!("{}.tfmt", name);
    let scripts = get_all_scripts();
    // These were selected through path.is_file(), unwrap should be safe.
    scripts
        .into_iter()
        .find(|p| p.file_name().unwrap() == name.as_str())
        .ok_or_else(|| anyhow!("Unable to find script {}", name))
}
