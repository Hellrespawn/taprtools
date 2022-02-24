use anyhow::{bail, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};
use test_harness::test_runner;
use tfmttools::cli::Args;

type CdFunction = fn(&Path) -> ();
type Mutex = std::sync::Mutex<CdFunction>;
type MutexGuard<'a> = std::sync::MutexGuard<'a, CdFunction>;

// Controls access to set_current_dir
static CD_MUTEX: Lazy<Mutex> = Lazy::new(|| {
    Mutex::new(|p: &Path| {
        std::env::set_current_dir(p).expect("Unable to set current directory!")
    })
});

const DEFAULT_RECURSION_DEPTH: usize = 4;
const INITIAL_REFERENCE: [&str; 5] = [
    "files/Dune - MASTER BOOT RECORD.mp3",
    "files/SET MIDI=SYNTH1 MAPG MODE1 - MASTER BOOT RECORD.mp3",
    "files/Under Siege - Amon Amarth.mp3",
    "files/Welcome To Heaven - Damjan Mravunac.ogg",
    "files/While Your Lips Are Still Red - Nightwish.mp3",
];
const SIMPLE_INPUT_REFERENCE: [&str; 5] = [
    "MASTER BOOT RECORD/Dune.mp3",
    "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
    "Amon Amarth/Under Siege.mp3",
    "Damjan Mravunac/Welcome To Heaven.ogg",
    "Nightwish/While Your Lips Are Still Red.mp3",
];
const TYPICAL_INPUT_REFERENCE: [&str; 5] = [
    "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
    "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
    "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
    "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
    "myname/Nightwish/While Your Lips Are Still Red.mp3",
];

const TYPICAL_INPUT_ARGS: &str = "myname";

struct TestEnv<'a> {
    tempdir: TempDir,
    previous_cwd: PathBuf,
    set_cwd: MutexGuard<'a>,
}

impl<'a> TestEnv<'a> {
    const CONFIG_FOLDER: &'static str = "config";
    const FILES_FOLDER: &'static str = "files";

    fn init() -> Result<Self> {
        let env = TestEnv {
            tempdir: Builder::new().prefix("tfmttools-").tempdir()?,
            previous_cwd: std::env::current_dir()?,
            set_cwd: CD_MUTEX.lock().expect("Unable to acquire lock on mutex!"),
        };

        env.populate_scripts()?;
        env.populate_files()?;

        env.set_cwd();

        Ok(env)
    }

    fn populate_scripts(&self) -> Result<()> {
        let paths: Vec<PathBuf> = std::fs::read_dir("testdata/script")?
            .flat_map(|r| r.map(|d| d.path()))
            .collect();

        std::fs::create_dir(self.get_config_dir())?;

        for script_path in &paths {
            // Scripts are selected by is_file, should always have a filename so
            // path.file_name().unwrap() should be safe.

            assert!(script_path.file_name().is_some());
            let file_name = script_path.file_name().unwrap();

            std::fs::copy(script_path, self.get_config_dir().join(file_name))?;
        }

        Ok(())
    }

    fn populate_files(&self) -> Result<()> {
        let paths: Vec<PathBuf> = std::fs::read_dir("testdata/music")?
            .flat_map(|r| r.map(|d| d.path()))
            .collect();

        std::fs::create_dir(self.get_files_dir())?;

        for audiofile_path in &paths {
            // Audio files are selected by is_file, should always have a
            // filename so path.file_name().unwrap() should be safe.

            assert!(audiofile_path.file_name().is_some());

            std::fs::copy(
                audiofile_path,
                self.get_files_dir()
                    .join(audiofile_path.file_name().unwrap()),
            )?;
        }

        assert!(check_paths(self.path(), &INITIAL_REFERENCE).is_ok());

        Ok(())
    }

    fn path(&self) -> &Path {
        self.tempdir.path()
    }

    fn set_cwd(&self) {
        (*self.set_cwd)(self.path())
    }

    fn restore_cwd(&self) {
        (*self.set_cwd)(&self.previous_cwd)
    }

    fn get_config_dir(&self) -> PathBuf {
        self.path().join(TestEnv::CONFIG_FOLDER)
    }

    fn get_files_dir(&self) -> PathBuf {
        self.path().join(TestEnv::FILES_FOLDER)
    }
}

fn setup_environment<'a>() -> Result<TestEnv<'a>> {
    TestEnv::init()
}

fn teardown_environment(env: TestEnv) -> Result<()> {
    // Must do this, otherwise we can't close the tempdir.
    env.restore_cwd();
    std::fs::copy(
        &env.get_config_dir().join("tfmttools.hist"),
        "d:\\histfile",
    )?;

    // Want to be explicit about dropping the guard.
    std::mem::drop(env.set_cwd);
    env.tempdir.close()?;
    Ok(())
}

fn print_filetree(path: &Path, depth: usize) {
    let components: Vec<_> = path.iter().flat_map(|o| o.to_str()).collect();
    let start = components.len() - depth - 1;
    let display_path =
        components[start..].join(&std::path::MAIN_SEPARATOR.to_string());

    println!("{}{}", display_path, std::path::MAIN_SEPARATOR);

    if depth == DEFAULT_RECURSION_DEPTH {
        return;
    }

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
                            "{}{}{}",
                            display_path,
                            std::path::MAIN_SEPARATOR,
                            p.file_name().unwrap().to_string_lossy()
                        );
                    }
                }
            }
        }
    }
}

fn check_paths<P>(root: &Path, reference: &[P]) -> Result<()>
where
    P: AsRef<Path>,
{
    for r in reference {
        let path = root.join(r);

        if !path.is_file() {
            print_filetree(root, 0);
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

    check_paths(env.path(), reference)?;

    Ok(())
}

fn test_undo(script_name: &str, args: &str, env: &TestEnv) -> Result<()> {
    test_rename(script_name, args, &TYPICAL_INPUT_REFERENCE, env)?;

    let args = format!(
        "tfmttools_test --config-folder {} undo",
        env.get_config_dir().display(),
    );

    let parsed_args = parse_custom_args(&args);

    tfmttools::cli::with_custom_args(parsed_args)?;

    check_paths(env.path(), &INITIAL_REFERENCE)?;

    Ok(())
}

fn test_redo(script_name: &str, args: &str, env: &TestEnv) -> Result<()> {
    test_undo(script_name, args, env)?;

    let args = format!(
        "tfmttools_test --config-folder {} redo",
        env.get_config_dir().display(),
    );

    let parsed_args = parse_custom_args(&args);

    tfmttools::cli::with_custom_args(parsed_args)?;

    check_paths(env.path(), &TYPICAL_INPUT_REFERENCE)?;

    Ok(())
}

#[test]
fn test_rename_simple_input() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        test_rename("simple_input", "", &SIMPLE_INPUT_REFERENCE, env)
    })
}

#[test]
fn test_rename_typical_input() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        test_rename(
            "typical_input",
            TYPICAL_INPUT_ARGS,
            &TYPICAL_INPUT_REFERENCE,
            env,
        )
    })
}

#[test]
fn test_undo_typical_input() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        test_undo("typical_input", TYPICAL_INPUT_ARGS, env)
    })
}

#[test]
fn test_redo_typical_input() -> Result<()> {
    test_runner(setup_environment, teardown_environment, |env| {
        test_redo("typical_input", TYPICAL_INPUT_ARGS, env)
    })
}

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
