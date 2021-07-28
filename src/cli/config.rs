use anyhow::{anyhow, Result};
use log::debug;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn get_log_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("tfmttools");
    path
}
fn get_config_dirs() -> &'static [PathBuf] {
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
            Ok(_) => vec![Some(PathBuf::from("testdata")), cwd, home, config],
            Err(_) => vec![cwd, home, config],
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

fn search_dir_for_script(dir: &Path) -> HashMap<String, PathBuf> {
    let mut scripts = HashMap::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(extension) = path.extension() {
                        // TODO? Mime type or something?
                        if extension == "tfmt" {
                            scripts.insert(
                                // FIXME Handle this unwrap
                                path.file_stem()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string(),
                                path,
                            );
                        }
                    }
                    // TODO? Do we want recursion?
                    // } else if file_type.is_dir() {
                    //     scripts.extend(search_dir_for_script(&path))
                }
            }
        }
    }

    scripts
}

pub fn get_all_scripts() -> HashMap<String, PathBuf> {
    let mut scripts = HashMap::new();

    for dir in get_config_dirs() {
        let mut dir = PathBuf::from(dir);
        scripts.extend(search_dir_for_script(&dir));

        dir.push("script");
        scripts.extend(search_dir_for_script(&dir));
        dir.pop();

        dir.push("scripts");
        scripts.extend(search_dir_for_script(&dir));
    }

    debug!("Found scripts:\n{:#?}", scripts);

    scripts
}

pub fn get_script(name: &str) -> Result<PathBuf> {
    get_all_scripts()
        .get(name)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("Unable to read script {}!", name))
}
