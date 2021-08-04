use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use tfmttools::cli::tfmt;

const CONFIG_FOLDER: &str = "config";
const SOURCE_FOLDER: &str = "source";

fn setup_environment(suffix: &str) -> Result<TempDir> {
    let tempdir = Builder::new()
        .prefix("tfmttools-")
        .suffix(&("-".to_string() + suffix))
        .tempdir()?;

    let path = tempdir.as_ref();

    std::fs::create_dir_all(path.join("0"))?;

    println!(r#"Temporary directory at "{}""#, path.to_string_lossy());

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

fn test_unrelated_dirs(path: &Path) {
    assert!(path.join("0").is_dir());
    assert!(path.join(CONFIG_FOLDER).join("1").is_dir());
}

fn teardown_environment(tempdir: TempDir) -> Result<()> {
    test_unrelated_dirs(tempdir.path());
    tempdir.close()?;

    Ok(())
}

fn print_filetree(path: &Path, depth: u64) {
    println!(
        "{}{}{}",
        std::iter::repeat(' ')
            .take((4 * depth) as usize)
            .collect::<String>(),
        path.file_name().unwrap().to_string_lossy(),
        std::path::MAIN_SEPARATOR
    );

    if let Ok(rd) = std::fs::read_dir(path) {
        for d in rd {
            match d {
                Err(_) => continue,
                Ok(d) => {
                    let p = d.path();

                    if p.is_dir() {
                        print_filetree(&p, depth + 1);
                    } else if p.is_file() {
                        println!(
                            "{}{}",
                            std::iter::repeat(' ')
                                .take((4 * depth + 4) as usize)
                                .collect::<String>(),
                            p.file_name().unwrap().to_string_lossy()
                        );
                    }
                }
            }
        }
    }
}

fn check_paths<P: AsRef<Path>>(
    tempdir: &TempDir,
    reference: &[P],
) -> Result<()> {
    for r in reference {
        let path = tempdir.path().join(r);

        if !path.is_file() {
            print_filetree(&tempdir.as_ref(), 0);
            bail!("File {} not in expected place!", path.to_string_lossy())
        }
    }

    Ok(())
}

fn test_rename<P: AsRef<Path>>(
    name: &str,
    args: &str,
    reference: &[P],
    tempdir: &TempDir,
) -> Result<()> {
    let args = format!(
        "tfmttools_test --config-folder {} rename {} --input-folder {} --output-folder {} -r {}",
        tempdir.path().join(CONFIG_FOLDER).to_string_lossy(),
        name,
        tempdir.path().join(SOURCE_FOLDER).to_string_lossy(),
        tempdir.path().to_string_lossy(),
        args
    );

    tfmt::main(&args.split_whitespace().collect::<Vec<&str>>())?;

    check_paths(&tempdir, &reference)?;

    Ok(())
}

fn test_undo<P: AsRef<Path>>(
    name: &str,
    args: &str,
    reference: &[P],
    tempdir: &TempDir,
) -> Result<()> {
    test_rename(name, args, reference, tempdir)?;

    let args = format!(
        "tfmttools_test --config-folder {} undo",
        tempdir.path().join(CONFIG_FOLDER).to_string_lossy(),
    );

    tfmt::main(&args.split_whitespace().collect::<Vec<&str>>())?;

    let reference = [
        "source/Dune - MASTER BOOT RECORD.mp3",
        "source/SET MIDI=SYNTH1 MAPG MODE1 - MASTER BOOT RECORD.mp3",
        "source/Under Siege - Amon Amarth.mp3",
        "source/Welcome To Heaven - Damjan Mravunac.ogg",
        "source/While Your Lips Are Still Red - Nightwish.mp3",
    ];

    check_paths(&tempdir, &reference)?;

    Ok(())
}

fn test_redo<P: AsRef<Path>>(
    name: &str,
    args: &str,
    reference: &[P],
    tempdir: &TempDir,
) -> Result<()> {
    test_undo(name, args, reference, tempdir)?;

    let args = format!(
        "tfmttools_test --config-folder {} redo",
        tempdir.path().join(CONFIG_FOLDER).to_string_lossy(),
    );

    tfmt::main(&args.split_whitespace().collect::<Vec<&str>>())?;

    check_paths(&tempdir, &reference)?;

    Ok(())
}

#[test]
fn tfmttools_simple_input_test() -> Result<()> {
    let name = "simple_input";
    let tempdir = setup_environment(name)?;

    let args = "";

    let reference = [
        "MASTER BOOT RECORD/Dune.mp3",
        "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "Amon Amarth/Under Siege.mp3",
        "Damjan Mravunac/Welcome To Heaven.ogg",
        "Nightwish/While Your Lips Are Still Red.mp3",
    ];

    match test_rename(name, args, &reference, &tempdir) {
        Ok(()) => Ok(teardown_environment(tempdir)?),
        Err(err) => bail!("Error in {}:\n{}", name, err),
    }
}

#[test]
fn tfmttools_typical_input_test() -> Result<()> {
    let name = "typical_input";
    let tempdir = setup_environment(name)?;

    let args = "-- myname";

    let reference = [
        "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
        "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
        "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
        "myname/Nightwish/While Your Lips Are Still Red.mp3",
    ];

    match test_rename(name, args, &reference, &tempdir) {
        Ok(()) => Ok(teardown_environment(tempdir)?),
        Err(err) => bail!("Error in {}:\n{}", name, err),
    }
}

#[test]
fn tfmttools_undo_test() -> Result<()> {
    let name = "typical_input";
    let tempdir = setup_environment(name)?;

    let args = "-- myname";

    let reference = [
        "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
        "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
        "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
        "myname/Nightwish/While Your Lips Are Still Red.mp3",
    ];

    match test_undo(name, args, &reference, &tempdir) {
        Ok(()) => Ok(teardown_environment(tempdir)?),
        Err(err) => bail!("Error in {}:\n{}", name, err),
    }
}

#[test]
fn tfmttools_redo_test() -> Result<()> {
    let name = "typical_input";
    let tempdir = setup_environment(name)?;

    let args = "-vvvvv -- myname";

    let reference = [
        "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
        "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
        "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
        "myname/Nightwish/While Your Lips Are Still Red.mp3",
    ];

    match test_redo(name, args, &reference, &tempdir) {
        Ok(()) => Ok(teardown_environment(tempdir)?),
        Err(err) => bail!("Error in {}:\n{}", name, err),
    }
}
