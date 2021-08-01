use super::rename::PathPairs;
use anyhow::{bail, Result};
use log::warn;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn validate<P: AsRef<Path>>(
    paths: &[(P, P)],
) -> Result<(PathPairs, PathPairs)> {
    // TODO? Extended print output
    validate_collisions(paths)?;
    validate_existing_files(paths)?;
    validate_movement(paths)
}

fn validate_collisions<P: AsRef<Path>>(paths: &[(P, P)]) -> Result<()> {
    let mut map = HashMap::new();

    paths.iter().for_each(|(orig, dest)| {
        map.entry(dest.as_ref())
            .or_insert_with(Vec::new)
            .push(orig.as_ref())
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
) -> Result<(PathPairs, PathPairs)> {
    let (no_move, to_move): (PathPairs, PathPairs) = paths
        .iter()
        .map(|(orig, dest)| {
            (PathBuf::from(orig.as_ref()), PathBuf::from(dest.as_ref()))
        })
        .partition(|(orig, dest)| orig == dest);

    Ok((to_move, no_move))
}
