use anyhow::{anyhow, Result};
use log::debug;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Get the default logging directory.
pub fn get_log_dir() -> PathBuf {
    std::env::temp_dir().join("tfmttools")
}

/// Search a path for files matching `predicate`, recursing for `depth`.
pub fn search_path<P, Q>(path: &P, predicate: Q, depth: u64) -> Vec<PathBuf>
where
    P: AsRef<Path>,
    // TODO Find out why Copy is necessary.
    Q: Copy + Fn(&Path) -> bool,
{
    if depth == 0 {
        return Vec::new();
    }

    let mut found_paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(path) {
        for entry in iter.flatten() {
            let entry_path = entry.path();

            if entry_path.is_file() && predicate(&entry_path) {
                found_paths.push(entry_path)
            } else if entry_path.is_dir() {
                found_paths.extend(search_path(
                    &entry_path,
                    predicate,
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

    let condition = |p: &Path| p.extension().unwrap_or_default() == "tfmt";

    if let Ok(cwd) = std::env::current_dir() {
        scripts.extend(search_path(&cwd, condition, 1));
    }

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
    let name = if !name.ends_with(".tfmt") {
        format!("{}.tfmt", name)
    } else {
        name.to_string()
    };

    let name_as_path = PathBuf::from(&name);

    if name_as_path.is_file() {
        return Ok(name_as_path);
    }

    let scripts = get_scripts(config_folder);
    scripts
        .into_iter()
        .find(|p| {
            // These were selected through p.is_file(), unwrap should be safe.
            debug_assert!(p.is_file());
            p.file_name().unwrap() == name.as_str()
        })
        .ok_or_else(|| anyhow!("Unable to find script {}", name))
}

/// Normalizes newlines in `string`.
pub fn normalize_newlines<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace("\r\n", "\n").replace("\r", "\n")
}

/// Normalizes separators for the platform in `string`.
pub fn normalize_separators<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace(
        if MAIN_SEPARATOR == '/' { '\\' } else { '/' },
        &MAIN_SEPARATOR.to_string(),
    )
}

/// Preview prefix
pub fn pp(preview: bool) -> &'static str {
    if preview {
        "[P] "
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_normalize_test() {
        let input =
            "This \n string \r has \r\n CRs and \r\r\n\n LFs mixed together!";
        assert_eq!(
            normalize_newlines(&input),
            "This \n string \n has \n CRs and \n\n\n LFs mixed together!"
        );
    }
}
