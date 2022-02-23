use anyhow::Result;
use std::path::PathBuf;
use tempfile::{Builder, TempDir};
use test_harness::{none, test_runner};

const CONFIG_FOLDER: &str = "config";
const SOURCE_FOLDER: &str = "source";

fn setup_environment() -> Result<TempDir> {
    let tempdir = Builder::new().prefix("tfmttools-").tempdir()?;

    let path = tempdir.path();

    std::fs::create_dir_all(path)?;

    println!(r#"Temporary directory at "{}""#, path.display());

    // Create script files
    let script_paths: Vec<PathBuf> = std::fs::read_dir("testdata/script")?
        .flat_map(|r| r.map(|d| d.path()))
        .collect();

    std::fs::create_dir_all(path.join(CONFIG_FOLDER))?;

    for script_path in &script_paths {
        // Scripts are selected by is_file, should always have a filename so
        // path.file_name().unwrap() should be safe.

        assert!(script_path.file_name().is_some());

        std::fs::copy(
            script_path,
            path.join(CONFIG_FOLDER)
                .join(script_path.file_name().unwrap()),
        )?;
    }

    std::fs::create_dir_all(path.join(CONFIG_FOLDER).join("1"))?;

    let audio_file_paths: Vec<PathBuf> = std::fs::read_dir("testdata/music")?
        .flat_map(|r| r.map(|d| d.path()))
        .collect();

    // Create audio files
    std::fs::create_dir_all(path.join(SOURCE_FOLDER))?;
    for audio_file_path in &audio_file_paths {
        // Audio files are selected by is_file, should always have a filename so
        // path.file_name().unwrap() should be safe.

        assert!(audio_file_path.file_name().is_some());

        std::fs::copy(
            audio_file_path,
            path.join(SOURCE_FOLDER)
                .join(audio_file_path.file_name().unwrap()),
        )?;
    }

    Ok(tempdir)
}
