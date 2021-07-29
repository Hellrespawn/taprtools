use anyhow::{anyhow, bail, Result};
use log::debug;
use std::path::{Path, PathBuf};
use super::argparse::Args;

pub fn get_log_dir() -> PathBuf {
    std::env::temp_dir().join("tfmttools")
}

pub fn get_config_folder(args: &Args) -> Result<PathBuf> {
    let dir = if let Some(config_folder) = &args.config_folder {
        Ok(config_folder.to_path_buf())
    } else {
        dirs::home_dir().
        map(|p| p.join(".tfmttools"))
        .or_else(||
            dirs::config_dir()
            .map(|p| p.join("tfmttools"))
        ).ok_or_else(|| anyhow!("Unable to find valid configuration directory!"))
    }?;

    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    } else if !dir.is_dir() {
        bail!("{} is not a folder!", dir.to_string_lossy())
    }

    //FIXME Add testdata somehow.
    //let dirs: Vec<PathBuf> = if cfg!(test) || std::env::var("CARGO_HOME").is_ok() {
    //let dirs: Vec<PathBuf> = if cfg!(test) {

    Ok(dir)
}

// TODO? Join search_dir function, take a closure?
fn search_dir_for_extension<P: AsRef<Path>>(
    dir: &P,
    extension: &str,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(found_ext) = path.extension() {
                        // TODO? Mime type or something?
                        if found_ext == extension {
                            paths.push(path);
                        }
                    }
                    // TODO? Do we want recursion?
                    // } else if file_type.is_dir() {
                    //     paths.extend(search_dir_for_extension(&path))
                }
            }
        }
    }

    paths
}

pub fn search_dir_for_filename<P: AsRef<Path>>(
    dir: &P,
    filename: &str,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(iter) = std::fs::read_dir(dir) {
        for entry in iter.flatten() {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(found_fn) = path.file_name() {
                        // TODO? Mime type or something?
                        if found_fn == filename {
                            paths.push(path);
                        }
                    }
                    // TODO? Do we want recursion?
                    // } else if file_type.is_dir() {
                    //     paths.extend(search_dir_for_filename(&path))
                }
            }
        }
    }

    paths
}

pub fn get_all_scripts<P: AsRef<Path>>(config_folder: &P) -> Vec<PathBuf> {
    let mut scripts = Vec::new();

    let config_folder = config_folder.as_ref();

    scripts.extend(search_dir_for_extension(&config_folder, "tfmt"));
    scripts.extend(search_dir_for_extension(&config_folder.join("script"), "tfmt"));
    scripts.extend(search_dir_for_extension(&config_folder.join("scripts"), "tfmt"));

    debug!("Found scripts:\n{:#?}", scripts);
    scripts
}

pub fn get_script<P: AsRef<Path>>(name: &str, config_folder: &P) -> Result<PathBuf> {
    let name = format!("{}.tfmt", name);
    let scripts = get_all_scripts(config_folder);
    // These were selected through path.is_file(), unwrap should be safe.
    scripts
        .into_iter()
        .find(|p| p.file_name().unwrap() == name.as_str())
        .ok_or_else(|| anyhow!("Unable to find script {}", name))
}
