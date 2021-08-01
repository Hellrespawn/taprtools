use anyhow::{anyhow, Result};
use log::debug;
use std::path::{Path, PathBuf};

pub fn get_log_dir() -> PathBuf {
    std::env::temp_dir().join("tfmttools")
}

pub fn search_dir<P: AsRef<Path>>(
    dir: &P,
    condition: fn(&Path) -> bool,
    depth: u64,
) -> Vec<PathBuf> {
    if depth == 0 {
        return Vec::new();
    }

    let mut paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();

            if path.is_file() {
                if condition(&path) {
                    paths.push(path)
                }
            } else if path.is_dir() {
                paths.extend(search_dir(&path, condition, depth - 1))
            }
        }
    }

    paths
}

pub fn get_all_scripts<P: AsRef<Path>>(config_folder: &P) -> Vec<PathBuf> {
    let mut scripts = Vec::new();

    let config_folder = config_folder.as_ref();

    // This condition is only called if p.is_file() is true, so
    // p.extension().unwrap() should be safe.
    let closure = |p: &Path| {
        debug_assert!(p.is_file());
        p.extension().unwrap() == "tfmt"
    };

    scripts.extend(search_dir(&config_folder, closure, 1));
    scripts.extend(search_dir(&config_folder.join("script"), closure, 1));
    scripts.extend(search_dir(&config_folder.join("scripts"), closure, 1));

    debug!("Found scripts:\n{:#?}", scripts);
    scripts
}

pub fn get_script<P: AsRef<Path>>(
    name: &str,
    config_folder: &P,
) -> Result<PathBuf> {
    let name = format!("{}.tfmt", name);
    let scripts = get_all_scripts(config_folder);
    // These were selected through path.is_file(), unwrap should be safe.
    scripts
        .into_iter()
        .find(|p| {
            debug_assert!(p.is_file());
            p.file_name().unwrap() == name.as_str()
        })
        .ok_or_else(|| anyhow!("Unable to find script {}", name))
}

#[cfg(feature = "slow-progress-bars")]
pub fn sleep() {
    std::thread::sleep(std::time::Duration::from_millis(200));
}

#[cfg(not(feature = "slow-progress-bars"))]
pub fn sleep() {}
