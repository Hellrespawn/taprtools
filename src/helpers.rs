use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Get the default logging directory.
pub fn get_log_dir() -> PathBuf {
    todo!();
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
                found_paths.push(entry_path);
            } else if entry_path.is_dir() {
                found_paths.extend(search_path(
                    &entry_path,
                    predicate,
                    depth - 1,
                ));
            }
        }
    }

    found_paths
}

/// Normalizes newlines in `string`.
pub fn normalize_newlines<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalizes separators for the platform in `string`.
pub fn normalize_separators<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace(
        if MAIN_SEPARATOR == '/' { '\\' } else { '/' },
        &MAIN_SEPARATOR.to_string(),
    )
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
