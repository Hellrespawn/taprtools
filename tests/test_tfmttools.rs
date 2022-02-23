use anyhow::{bail, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use test_harness::test_runner;
use tfmttools::cli::Args;

const DEFAULT_RECURSION_DEPTH: usize = 4;

struct TestEnv {
    tempdir: TempDir,
}

impl TestEnv {
    const CONFIG_FOLDER: &'static str = "config";
    const FILES_FOLDER: &'static str = "files";

    fn new() -> Result<Self> {
        let tempdir = Builder::new().prefix("tfmttools-").tempdir()?;

        let env = TestEnv { tempdir };

        std::fs::create_dir(env.get_config_dir())?;
        std::fs::create_dir(env.get_files_dir())?;

        Ok(env)
    }

    fn path(&self) -> &Path {
        self.tempdir.path()
    }

    fn get_config_dir(&self) -> PathBuf {
        self.tempdir.path().join(TestEnv::CONFIG_FOLDER)
    }

    fn get_files_dir(&self) -> PathBuf {
        self.tempdir.path().join(TestEnv::FILES_FOLDER)
    }

    fn get_script_paths(&self) -> Result<Vec<PathBuf>> {
        // This only retrieves the contents of testdata/script, it does not check the files.
        let paths = std::fs::read_dir("testdata/script")?
            .flat_map(|r| r.map(|d| d.path()))
            .collect();

        Ok(paths)
    }

    fn get_audiofile_paths(&self) -> Result<Vec<PathBuf>> {
        // This only retrieves the contents of testdata/music, it does not check the files.
        let paths = std::fs::read_dir("testdata/music")?
            .flat_map(|r| r.map(|d| d.path()))
            .collect();

        Ok(paths)
    }
}

fn setup_environment() -> Result<TestEnv> {
    let env = TestEnv::new()?;

    for script_path in &env.get_script_paths()? {
        // Scripts are selected by is_file, should always have a filename so
        // path.file_name().unwrap() should be safe.

        assert!(script_path.file_name().is_some());

        std::fs::copy(
            script_path,
            env.get_config_dir().join(script_path.file_name().unwrap()),
        )?;
    }
    for audiofile_path in &env.get_audiofile_paths()? {
        // Audio files are selected by is_file, should always have a filename so
        // path.file_name().unwrap() should be safe.

        assert!(audiofile_path.file_name().is_some());

        std::fs::copy(
            audiofile_path,
            env.get_files_dir()
                .join(audiofile_path.file_name().unwrap()),
        )?;
    }

    std::env::set_current_dir(env.path())?;

    Ok(env)
}

fn teardown_environment(env: TestEnv) -> Result<()> {
    env.tempdir.close()?;
    Ok(())
}

fn print_filetree(path: &Path, depth: usize) {
    println!(
        "{}{}{}",
        std::iter::repeat(' ')
            .take((4 * depth) as usize)
            .collect::<String>(),
        path.file_name().unwrap().to_string_lossy(),
        std::path::MAIN_SEPARATOR
    );

    if depth == 0 {
        return;
    }

    if let Ok(rd) = std::fs::read_dir(path) {
        for d in rd {
            match d {
                Err(_) => continue,
                Ok(d) => {
                    let p = d.path();

                    if p.is_dir() {
                        print_filetree(&p, depth - 1);
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

fn check_paths<P>(path: &Path, reference: &[P]) -> Result<()>
where
    P: AsRef<Path>,
{
    for r in reference {
        let path = path.join(r);

        if !path.is_file() {
            print_filetree(&path, DEFAULT_RECURSION_DEPTH);
            bail!("File {} not in expected place!", path.display())
        }
    }

    Ok(())
}

fn parse_custom_args(args: &str) -> Args {
    Args::parse_from(args.split_whitespace().collect::<Vec<&str>>())
        .aggregate_preview(false)
}

fn test_rename<P>(
    script_name: &str,
    args: &str,
    reference: &[P],
    env: &TestEnv,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let args = format!(
        "tfmttools_test --config-folder {} rename {} {}",
        env.get_config_dir().display(),
        script_name,
        args
    );

    let parsed_args = parse_custom_args(&args);

    tfmttools::cli::with_custom_args(parsed_args)?;

    check_paths(env.path(), &reference)?;

    Ok(())
}

#[test]
fn test_environment() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        print_filetree(env.path(), DEFAULT_RECURSION_DEPTH);
        Ok(())
    })
}
// fn test_undo<P: AsRef<Path>>(
//     name: &str,
//     args: &str,
//     reference: &[P],
//     tempdir: &TempDir,
// ) -> Result<()> {
//     test_rename(name, args, reference, tempdir)?;

//     let args = format!(
//         "tfmttools_test --config-folder {} undo",
//         tempdir.path().join(CONFIG_FOLDER).display(),
//     );

//     // FIXME Rename here
//     // tfmt::main(&args.split_whitespace().collect::<Vec<&str>>(), false)?;

//     let reference = [
//         "source/Dune - MASTER BOOT RECORD.mp3",
//         "source/SET MIDI=SYNTH1 MAPG MODE1 - MASTER BOOT RECORD.mp3",
//         "source/Under Siege - Amon Amarth.mp3",
//         "source/Welcome To Heaven - Damjan Mravunac.ogg",
//         "source/While Your Lips Are Still Red - Nightwish.mp3",
//     ];

//     check_paths(&tempdir, &reference)?;

//     Ok(())
// }

// fn test_redo<P: AsRef<Path>>(
//     name: &str,
//     args: &str,
//     reference: &[P],
//     tempdir: &TempDir,
// ) -> Result<()> {
//     test_undo(name, args, reference, tempdir)?;

//     let args = format!(
//         "tfmttools_test --config-folder {} redo",
//         tempdir.path().join(CONFIG_FOLDER).display(),
//     );

//     // FIXME Rename here
//     // tfmt::main(&args.split_whitespace().collect::<Vec<&str>>(), false)?;

//     check_paths(&tempdir, &reference)?;

//     Ok(())
// }

#[test]
fn test_simple_input() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        let script_name = "simple_input";

        let args = "";

        let reference = [
            "MASTER BOOT RECORD/Dune.mp3",
            "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
            "Amon Amarth/Under Siege.mp3",
            "Damjan Mravunac/Welcome To Heaven.ogg",
            "Nightwish/While Your Lips Are Still Red.mp3",
        ];

        let result = test_rename(script_name, args, &reference, env);

        if result.is_err() {
            print_filetree(env.path(), DEFAULT_RECURSION_DEPTH)
        }

        result
    })
}

// #[test]
// fn tfmttools_typical_input_test() -> Result<()> {
//     let name = "typical_input";
//     let tempdir = setup_environment(name)?;

//     let args = "-- myname";

//     let reference = [
//         "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
//         "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
//         "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
//         "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
//         "myname/Nightwish/While Your Lips Are Still Red.mp3",
//     ];

//     match test_rename(name, args, &reference, &tempdir) {
//         Ok(()) => Ok(teardown_environment(tempdir)?),
//         Err(err) => bail!("Error in {}:\n{}", name, err),
//     }
// }

// #[test]
// fn tfmttools_undo_test() -> Result<()> {
//     let name = "typical_input";
//     let tempdir = setup_environment(name)?;

//     let args = "-- myname";

//     let reference = [
//         "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
//         "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
//         "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
//         "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
//         "myname/Nightwish/While Your Lips Are Still Red.mp3",
//     ];

//     match test_undo(name, args, &reference, &tempdir) {
//         Ok(()) => Ok(teardown_environment(tempdir)?),
//         Err(err) => bail!("Error in {}:\n{}", name, err),
//     }
// }

// #[test]
// fn tfmttools_redo_test() -> Result<()> {
//     let name = "typical_input";
//     let tempdir = setup_environment(name)?;

//     let args = "-vvvvv -- myname";

//     let reference = [
//         "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
//         "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
//         "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
//         "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
//         "myname/Nightwish/While Your Lips Are Still Red.mp3",
//     ];

//     match test_redo(name, args, &reference, &tempdir) {
//         Ok(()) => Ok(teardown_environment(tempdir)?),
//         Err(err) => bail!("Error in {}:\n{}", name, err),
//     }
// }

// #[test]
// fn tfmttools_collision_test() -> Result<()> {
//     let name = "collisions";
//     let tempdir = setup_environment(name)?;

//     let args = "";

//     let reference = [""];

//     match test_rename(name, args, &reference, &tempdir) {
//         Err(err) if err.to_string().contains("collision was detected") => {
//             Ok(teardown_environment(tempdir)?)
//         }
//         Err(err) => bail!("Unexpected error in collision_test: {}", err),
//         Ok(()) => bail!("collision_test did not produce an error!"),
//     }
// }
