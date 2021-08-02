use super::rename::SrcTgtPair;
use anyhow::{bail, Result};
use log::warn;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Returns (to_move, no_move)
pub fn validate<P: AsRef<Path>>(
    paths: &[(P, P)],
) -> Result<(Vec<SrcTgtPair>, Vec<SrcTgtPair>)> {
    // TODO? Extended print output
    validate_collisions(paths)?;
    validate_existing_files(paths)?;
    validate_movement(paths)
}

fn validate_collisions<P: AsRef<Path>>(paths: &[(P, P)]) -> Result<()> {
    let mut map = HashMap::new();

    paths.iter().for_each(|(src, tgt)| {
        map.entry(tgt.as_ref())
            .or_insert_with(Vec::new)
            .push(src.as_ref())
    });

    let collisions: HashMap<&Path, Vec<&Path>> =
        map.into_iter().filter(|(_, v)| v.len() > 1).collect();

    if !collisions.is_empty() {
        //FIXME More extensive print
        let s = format!("{} collisions were detected!", collisions.len());
        //println!("{}", s);
        warn!("{}", s);
        bail!("{}", s)
    } else {
        Ok(())
    }
}

fn validate_existing_files<P: AsRef<Path>>(paths: &[(P, P)]) -> Result<()> {
    let existing: Vec<&Path> = paths
        .iter()
        .filter_map(|(_, dest)| dest.as_ref().exists().then(|| dest.as_ref()))
        .collect();
    if !existing.is_empty() {
        //FIXME yeah shorten this
        let s = format!(
            "The following file already exist:\n{}",
            existing
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<String>>()
                .join("\n")
        );
        warn!("{}", s);
        bail!("{}", s);
    }

    Ok(())
}
fn validate_movement<P: AsRef<Path>>(
    paths: &[(P, P)],
) -> Result<(Vec<SrcTgtPair>, Vec<SrcTgtPair>)> {
    let (no_move, to_move): (Vec<SrcTgtPair>, Vec<SrcTgtPair>) = paths
        .iter()
        .map(|(src, tgt)| {
            (PathBuf::from(src.as_ref()), PathBuf::from(tgt.as_ref()))
        })
        .partition(|(src, tgt)| src == tgt);

    Ok((to_move, no_move))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_collisions_test() -> Result<()> {
        let reference = [
            ("/a/b/c.file", "/b/c/d.file"),
            ("/c/d/e.file", "/b/c/d.file"),
        ];

        if let Ok(()) = validate_collisions(&reference) {
            bail!("validate_collisions should have returned an error!")
        }

        let reference = [
            ("/a/b/c.file", "/b/c/d.file"),
            ("/c/d/e.file", "/d/e/f.file"),
        ];

        if let Err(err) = validate_collisions(&reference) {
            bail!(
                "validate_collisions returned an error when it shouldn't!\n{}",
                err
            )
        }

        Ok(())
    }

    #[test]
    fn validate_movement_test() -> Result<()> {
        let paths = [
            ("/a/b/c.file", "/a/b/c.file"),
            ("/c/d/e.file", "/b/c/d.file"),
        ];

        let (to_move, no_move) = validate_movement(&paths)?;

        assert_eq!(
            to_move,
            [(PathBuf::from("/c/d/e.file"), PathBuf::from("/b/c/d.file"))]
        );
        assert_eq!(
            no_move,
            [(PathBuf::from("/a/b/c.file"), PathBuf::from("/a/b/c.file"))]
        );

        Ok(())
    }
}
