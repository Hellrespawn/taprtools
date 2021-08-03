use anyhow::{anyhow, Result};
use log::debug;
use std::path::{Path, PathBuf};

/// Get the default logging directory.
pub fn get_log_dir() -> PathBuf {
    std::env::temp_dir().join("tfmttools")
}

/// Search a path
pub fn search_path<P: AsRef<Path>>(
    path: &P,
    condition: fn(&Path) -> bool,
    depth: u64,
) -> Vec<PathBuf> {
    if depth == 0 {
        return Vec::new();
    }

    let mut found_paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(path) {
        for entry in iter.flatten() {
            let entry_path = entry.path();

            if entry_path.is_file() && condition(&entry_path) {
                found_paths.push(entry_path)
            } else if entry_path.is_dir() {
                found_paths.extend(search_path(
                    &entry_path,
                    condition,
                    depth - 1,
                ))
            }
        }
    }

    found_paths
}

/// Reads all scripts from `config_folder`.
pub fn get_scripts<P: AsRef<Path>>(config_folder: &P) -> Vec<PathBuf> {
    let mut scripts = Vec::new();

    let config_folder = config_folder.as_ref();

    // This condition is only called if p.is_file() is true, so
    // p.extension().unwrap() should be safe.
    let condition = |p: &Path| {
        debug_assert!(p.is_file());
        p.extension().unwrap() == "tfmt"
    };

    scripts.extend(search_path(&config_folder, condition, 1));
    scripts.extend(search_path(&config_folder.join("script"), condition, 1));
    scripts.extend(search_path(&config_folder.join("scripts"), condition, 1));

    debug!("Found scripts:\n{:#?}", scripts);
    scripts
}

/// Try to find script `name` in `config_folder`.
pub fn get_script<P: AsRef<Path>>(
    name: &str,
    config_folder: &P,
) -> Result<PathBuf> {
    let name = format!("{}.tfmt", name);
    let scripts = get_scripts(config_folder);
    // These were selected through path.is_file(), unwrap should be safe.
    scripts
        .into_iter()
        .find(|p| {
            debug_assert!(p.is_file());
            p.file_name().unwrap() == name.as_str()
        })
        .ok_or_else(|| anyhow!("Unable to find script {}", name))
}

/// Titlecases `string`.
pub fn titlecase(string: &str) -> String {
    let mut chars = string.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Normalizes newlines
pub fn normalize_newlines<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace("\r\n", "\n").replace("\r", "\n")
}

/// Preview Prefix
pub fn pp<'a>(preview: bool) -> &'a str {
    if preview {
        "[D] "
    } else {
        ""
    }
}

#[cfg(feature = "slow-progress-bars")]
/// Slows progress bars for testing.
pub fn sleep() {
    std::thread::sleep(std::time::Duration::from_millis(200));
}

#[cfg(not(feature = "slow-progress-bars"))]
/// No-op
pub fn sleep() {}
