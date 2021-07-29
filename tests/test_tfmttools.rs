use anyhow::{anyhow, bail, Result};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use tfmttools::cli::tfmttools;

struct Environment {
    tempdir: TempDir,
}
//FIXME Temp dirs clash?
fn setup_environment(suffix: &str) -> Result<Environment> {
    let environment = Environment {
        tempdir: Builder::new()
            .prefix("tfmttools-")
            .suffix(&("-".to_string() + suffix))
            .tempdir()?,
    };

    let path = environment.tempdir.as_ref();

    println!("Temporary directory at \"{}\"", path.to_string_lossy());

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

    Ok(environment)
}

fn teardown_environment(environment: Environment) -> Result<()> {
    environment.tempdir.close()?;

    Ok(())
}

fn print_filetree(path: &Path, depth: u64) {
    if depth > 3 {
        return;
    }
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
    environment: &Environment,
    reference: &[P],
) -> Result<()> {
    for r in reference {
        let path = environment.tempdir.path().join(r);

        if !path.is_file() {
            print_filetree(&environment.tempdir.path(), 0);
            bail!("File {} not in expected place!", path.to_string_lossy())
        }
    }

    Ok(())
}

fn test_tfmttools<P: AsRef<Path>>(
    name: &str,
    args: &str,
    reference: &[P],
) -> Result<()> {
    let environment = setup_environment(name)?;

    let args = format!(
        "tfmttools_test rename {0} --input-folder {1} --output-folder {1} --config-folder testdata -r {2}",
        name,
        environment.tempdir.path().to_string_lossy(),
        args
    );

    tfmttools::_main(&args.split_whitespace().collect::<Vec<&str>>())?;

    check_paths(&environment, &reference)?;

    teardown_environment(environment)?;

    Ok(())
}

#[test]
fn tfmttools_simple_input_test() -> Result<()> {
    let args = "";

    let reference = [
        "MASTER BOOT RECORD/Dune.mp3",
        "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "Amon Amarth/Under Siege.mp3",
        "Damjan Mravunac/Welcome To Heaven.ogg",
        "Nightwish/While Your Lips Are Still Red.mp3",
    ];

    test_tfmttools("simple_input", args, &reference)
        .map_err(|e| anyhow!("Error in simple_input:\n{}", e))
}

#[test]
fn tfmttools_typical_input_test() -> Result<()> {
    let args = "-vvvvv -- myname";

    let reference = [
        "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
        "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
        "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
        "myname/Nightwish/While Your Lips Are Still Red.mp3",
    ];

    test_tfmttools("typical_input", args, &reference)
        .map_err(|e| anyhow!("Error in typical_input:\n{}", e))
}
