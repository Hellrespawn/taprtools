use std::path::{Path, PathBuf, MAIN_SEPARATOR};

/// Search a path for files matching `predicate`, recursing for `depth`.
pub(crate) fn search_path<P, Q>(
    path: &P,
    depth: u64,
    predicate: Q,
) -> Vec<PathBuf>
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
            } else if entry_path.is_dir() {
                if depth > 0 {
                    found_paths.extend(search_path(
                        &entry_path,
                        depth - 1,
                        predicate,
                    ));
                }
            }
        }
    }

    found_paths
}

/// Normalizes newlines in `string`.
pub(crate) fn normalize_newlines<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace("\r\n", "\n").replace('\r', "\n")
}

/// Normalizes separators for the platform in `string`.
pub(crate) fn normalize_separators<S: AsRef<str>>(string: &S) -> String {
    string.as_ref().replace(
        if MAIN_SEPARATOR == '/' { '\\' } else { '/' },
        &MAIN_SEPARATOR.to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_newlines() {
        let input =
            "This \n string \r has \r\n CRs and \r\r\n\n LFs mixed together!";
        assert_eq!(
            normalize_newlines(&input),
            "This \n string \n has \n CRs and \n\n\n LFs mixed together!"
        );
    }

    #[test]
    fn test_normalize_separators() {
        let input = "/alpha/beta\\gamma\\delta";

        let reference = vec!["alpha", "beta", "gamma", "delta"]
            .join(&MAIN_SEPARATOR.to_string());

        assert_eq!(normalize_separators(&input), reference);
    }
}
