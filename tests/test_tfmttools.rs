use anyhow::{Result, bail};
use std::path::{PathBuf, Path};
use tempfile::{tempdir, TempDir};
use tfmttools::cli::tfmttools;

struct Environment {
    tempdir: TempDir,
    cwd: PathBuf,
}

fn setup_environment() -> Result<Environment> {
    let environment = Environment {
        tempdir: tempdir()?,
        cwd: std::env::current_dir()?,
    };

    let path = environment.tempdir.as_ref();

    let song_paths: Vec<PathBuf> = std::fs::read_dir("testdata/music")?
        .flat_map(|r| r.map(|d| d.path()))
        .collect();

    std::fs::create_dir_all(path.join("origin"))?;
    for song_path in &song_paths {
        // Unchecked unwrap, probably works.
        std::fs::copy(
            song_path,
            path.join("origin").join(song_path.file_name().unwrap()),
        )?;
    }

    let script_paths: Vec<PathBuf> = std::fs::read_dir("testdata/script")?
        .flat_map(|r| r.map(|d| d.path()))
        .collect();

    for script_path in &script_paths {
        // Unchecked unwrap, probably works.
        std::fs::copy(
            script_path,
            path.join(script_path.file_name().unwrap()),
        )?;
    }

    std::env::set_current_dir(path)?;

    Ok(environment)
}

fn teardown_environment(environment: Environment) -> Result<()> {
    std::env::set_current_dir(&environment.cwd)?;

    environment.tempdir.close()?;

    Ok(())
}

fn check_paths<P: AsRef<Path>>(environment: &Environment, reference: &[P]) -> Result<()> {
    for r in reference {
        let path = environment.tempdir.path().join(r);

        if !path.is_file() {
            bail!("File {} not in expected place!", path.to_string_lossy())
        }
    }

    Ok(())
}

fn test_tfmttools<P: AsRef<Path>>(args: &str, reference: &[P]) -> Result<()> {
    let e = setup_environment()?;

    tfmttools::_main(&args.split_whitespace()
    .collect::<Vec<&str>>())?;

    check_paths(&e, &reference)?;

    teardown_environment(e)?;

    Ok(())

}

#[test]
fn tfmttools_simple_input_test() -> Result<()> {

    let args = "tfmttools_test -vvvvv rename simple_input -r";

    let reference = [
    "MASTER BOOT RECORD/Dune.mp3",
    "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
    "Amon Amarth/Under Siege.mp3",
    "Damjan Mravunac/Welcome To Heaven.ogg",
    "Nightwish/While Your Lips Are Still Red.mp3",
    ];

    test_tfmttools(args, &reference)
}

#[test]
fn tfmttools_typical_input_test() -> Result<()> {

    let args = "tfmttools_test -vvvvv rename typical_input -r -- myname";

    let reference = [
        "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
        "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
        "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
        "myname/Nightwish/While Your Lips Are Still Red.mp3",
    ];

    test_tfmttools(args, &reference)
}
