use crate::cli::args::Args;
use crate::cli::commands::SrcTgtPair;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Returns (`to_move`, `no_move`)
pub(crate) fn validate(pairs: &[SrcTgtPair]) -> Result<()> {
    validate_collisions(pairs)?;
    validate_existing_files(pairs)?;

    Ok(())
}

fn validate_collisions(pairs: &[SrcTgtPair]) -> Result<()> {
    let mut map = HashMap::new();

    for SrcTgtPair { source, target } in pairs {
        map.entry(target).or_insert_with(Vec::new).push(source);
    }

    let collisions: HashMap<&PathBuf, Vec<&PathBuf>> =
        map.into_iter().filter(|(_, v)| v.len() > 1).collect();

    if collisions.is_empty() {
        Ok(())
    } else {
        let string = format_collisions(&collisions);
        bail!(string)
    }
}

fn format_collisions(collisions: &HashMap<&PathBuf, Vec<&PathBuf>>) -> String {
    let length = collisions.len();
    let mut string = format!(
        "{} collision{} {} detected{}:\n",
        length,
        if length > 1 { "s" } else { "" },
        if length > 1 { "were" } else { "was" },
        if length > Args::DEFAULT_PREVIEW_AMOUNT {
            format!("! Showing {}", Args::DEFAULT_PREVIEW_AMOUNT)
        } else {
            String::new()
        },
    );

    for (i, (path, collisions)) in collisions.iter().enumerate() {
        if i >= Args::DEFAULT_PREVIEW_AMOUNT {
            break;
        }
        let length = collisions.len();
        string += &format!(
            "{} is pointed to by {} file{}{}:\n",
            path.display(),
            length,
            if length > 1 { "s" } else { "" },
            if length > Args::DEFAULT_PREVIEW_AMOUNT {
                format!("! Showing {}", Args::DEFAULT_PREVIEW_AMOUNT)
            } else {
                String::new()
            },
        );

        for (i, path) in collisions.iter().enumerate() {
            if i >= Args::DEFAULT_PREVIEW_AMOUNT {
                break;
            }
            string += &format!("{}\n", path.display());
        }
        string += "\n";
    }

    string
}

fn validate_existing_files(pairs: &[SrcTgtPair]) -> Result<()> {
    let existing: Vec<&PathBuf> = pairs
        .iter()
        .filter_map(|SrcTgtPair { target, .. }| target.exists().then(|| target))
        .collect();

    let length = existing.len();

    if !existing.is_empty() {
        let string = format!(
            "{} file{} already exist{}{}:\n{}",
            length,
            if length > 1 { "s" } else { "" },
            if length > 1 { "" } else { "s" },
            if length > Args::DEFAULT_PREVIEW_AMOUNT {
                format!("! Showing {}", Args::DEFAULT_PREVIEW_AMOUNT)
            } else {
                String::new()
            },
            existing
                .iter()
                .take(Args::DEFAULT_PREVIEW_AMOUNT)
                .map(|p| p.display().to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );
        bail!(string);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_collisions_test() -> Result<()> {
        let reference = [
            ("/a/b/c.file", "/b/c/d.file"),
            ("/c/d/e.file", "/b/c/d.file"),
        ]
        .map(|(source, target)| SrcTgtPair {
            source: PathBuf::from(source),
            target: PathBuf::from(target),
        });

        if let Ok(()) = validate_collisions(&reference) {
            bail!("validate_collisions should have returned an error!")
        }

        let reference = [
            ("/a/b/c.file", "/b/c/d.file"),
            ("/c/d/e.file", "/d/e/f.file"),
        ]
        .map(|(source, target)| SrcTgtPair {
            source: PathBuf::from(source),
            target: PathBuf::from(target),
        });

        if let Err(err) = validate_collisions(&reference) {
            bail!(
                "validate_collisions returned an error when it shouldn't!\n{}",
                err
            )
        }

        Ok(())
    }
}
