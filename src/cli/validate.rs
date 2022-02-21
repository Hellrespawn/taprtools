// use crate::cli::Config;
// use anyhow::{bail, Result};
// use std::collections::HashMap;
// use std::path::{Path, PathBuf};

// type SrcTgtPair = (PathBuf, PathBuf);

// /// Returns (`to_move`, `no_move`)
// pub fn validate<P: AsRef<Path>>(
//     paths: &[(P, P)],
// ) -> Result<(Vec<SrcTgtPair>, Vec<SrcTgtPair>)> {
//     validate_collisions(paths)?;
//     validate_existing_files(paths)?;

//     Ok(partition_src_tgt_pairs(paths))
// }

// fn format_collisions(collisions: &HashMap<&Path, Vec<&Path>>) -> String {
//     let length = collisions.len();
//     let mut string = format!(
//         "{} collision{} {} detected{}:\n",
//         length,
//         if length > 1 { "s" } else { "" },
//         if length > 1 { "were" } else { "was" },
//         if length > Config::DEFAULT_PREVIEW_AMOUNT {
//             format!("! Showing {}", Config::DEFAULT_PREVIEW_AMOUNT)
//         } else {
//             String::new()
//         },
//     );

//     for (i, (path, collisions)) in collisions.iter().enumerate() {
//         if i >= Config::DEFAULT_PREVIEW_AMOUNT {
//             break;
//         }
//         let length = collisions.len();
//         string += &format!(
//             "{} is pointed to by {} file{}{}:\n",
//             path.display(),
//             length,
//             if length > 1 { "s" } else { "" },
//             if length > Config::DEFAULT_PREVIEW_AMOUNT {
//                 format!("! Showing {}", Config::DEFAULT_PREVIEW_AMOUNT)
//             } else {
//                 String::new()
//             },
//         );

//         for (i, path) in collisions.iter().enumerate() {
//             if i >= Config::DEFAULT_PREVIEW_AMOUNT {
//                 break;
//             }
//             string += &format!("{}\n", path.display());
//         }
//         string += "\n";
//     }

//     string
// }

// fn validate_collisions<P: AsRef<Path>>(paths: &[(P, P)]) -> Result<()> {
//     let mut map = HashMap::new();

//     for (src, tgt) in paths {
//         map.entry(tgt.as_ref())
//             .or_insert_with(Vec::new)
//             .push(src.as_ref());
//     }

//     let collisions: HashMap<&Path, Vec<&Path>> =
//         map.into_iter().filter(|(_, v)| v.len() > 1).collect();

//     if collisions.is_empty() {
//         Ok(())
//     } else {
//         let string = format_collisions(&collisions);
//         bail!(string)
//     }
// }

// fn validate_existing_files<P: AsRef<Path>>(paths: &[(P, P)]) -> Result<()> {
//     let existing: Vec<&Path> = paths
//         .iter()
//         .filter_map(|(_, dest)| dest.as_ref().exists().then(|| dest.as_ref()))
//         .collect();

//     let length = existing.len();

//     if !existing.is_empty() {
//         let string = format!(
//             "{} file{} already exist{}{}:\n{}",
//             length,
//             if length > 1 { "s" } else { "" },
//             if length > 1 { "" } else { "s" },
//             if length > Config::DEFAULT_PREVIEW_AMOUNT {
//                 format!("! Showing {}", Config::DEFAULT_PREVIEW_AMOUNT)
//             } else {
//                 String::new()
//             },
//             existing
//                 .iter()
//                 .take(Config::DEFAULT_PREVIEW_AMOUNT)
//                 .map(|p| p.display().to_string())
//                 .collect::<Vec<String>>()
//                 .join("\n")
//         );
//         bail!(string);
//     }

//     Ok(())
// }
// fn partition_src_tgt_pairs<P: AsRef<Path>>(
//     paths: &[(P, P)],
// ) -> (Vec<SrcTgtPair>, Vec<SrcTgtPair>) {
//     // "no_move" and "to_move" are just similar
//     #![allow(clippy::similar_names)]
//     let (no_move, to_move): (Vec<SrcTgtPair>, Vec<SrcTgtPair>) = paths
//         .iter()
//         .map(|(src, tgt)| {
//             (PathBuf::from(src.as_ref()), PathBuf::from(tgt.as_ref()))
//         })
//         .partition(|(src, tgt)| src == tgt);

//     (to_move, no_move)
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn validate_collisions_test() -> Result<()> {
//         let reference = [
//             ("/a/b/c.file", "/b/c/d.file"),
//             ("/c/d/e.file", "/b/c/d.file"),
//         ];

//         if let Ok(()) = validate_collisions(&reference) {
//             bail!("validate_collisions should have returned an error!")
//         }

//         let reference = [
//             ("/a/b/c.file", "/b/c/d.file"),
//             ("/c/d/e.file", "/d/e/f.file"),
//         ];

//         if let Err(err) = validate_collisions(&reference) {
//             bail!(
//                 "validate_collisions returned an error when it shouldn't!\n{}",
//                 err
//             )
//         }

//         Ok(())
//     }

//     #[test]
//     fn validate_movement_test() -> Result<()> {
//         let paths = [
//             ("/a/b/c.file", "/a/b/c.file"),
//             ("/c/d/e.file", "/b/c/d.file"),
//         ];

//         let (to_move, no_move) = partition_src_tgt_pairs(&paths);

//         assert_eq!(
//             to_move,
//             [(PathBuf::from("/c/d/e.file"), PathBuf::from("/b/c/d.file"))]
//         );
//         assert_eq!(
//             no_move,
//             [(PathBuf::from("/a/b/c.file"), PathBuf::from("/a/b/c.file"))]
//         );

//         Ok(())
//     }
// }
