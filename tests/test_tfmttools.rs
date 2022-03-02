use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use test_harness::test_runner;

const TEST_DATA_DIRECTORY: &str = "tests/testdata/";

const INITIAL_REFERENCE: [&str; 7] = [
    "files/Dune - MASTER BOOT RECORD.mp3",
    "files/SET MIDI=SYNTH1 MAPG MODE1 - MASTER BOOT RECORD.mp3",
    "files/Under Siege - Amon Amarth.mp3",
    "files/Welcome To Heaven - Damjan Mravunac.ogg",
    "files/While Your Lips Are Still Red - Nightwish.mp3",
    "config/simple_input.tfmt",
    "config/typical_input.tfmt",
];

const TYPICAL_INPUT_REFERENCE: [&str; 5] = [
    "myname/MASTER BOOT RECORD/WAREZ/Dune.mp3",
    "myname/MASTER BOOT RECORD/2016.03 - CEDIT AUTOEXEC.BAT/05 - SET MIDI=SYNTH1 MAPG MODE1.mp3",
    "myname/Amon Amarth/2013 - Deceiver of the Gods/105 - Under Siege.mp3",
    "myname/The Talos Principle/2015 - The Talos Principle OST/01 - Damjan Mravunac - Welcome To Heaven.ogg",
    "myname/Nightwish/While Your Lips Are Still Red.mp3",
];

struct TestEnv {
    tempdir: TempDir,
}

impl TestEnv {
    fn new() -> Result<Self> {
        let env = TestEnv {
            tempdir: TempDir::new()?,
        };

        dbg!(env.path());

        env.populate_scripts()?;
        env.populate_files()?;

        Ok(env)
    }

    fn populate_scripts(&self) -> Result<()> {
        let paths: Vec<PathBuf> =
            std::fs::read_dir(TestEnv::get_test_data_dir().join("script"))?
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
        let paths: Vec<PathBuf> =
            std::fs::read_dir(TestEnv::get_test_data_dir().join("music"))?
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

        self.assert_files(&INITIAL_REFERENCE);

        // assert!(check_paths(self.path(), &INITIAL_REFERENCE).is_ok());

        Ok(())
    }

    fn get_test_data_dir() -> PathBuf {
        PathBuf::from(TEST_DATA_DIRECTORY)
    }

    fn path(&self) -> &Path {
        self.tempdir.path()
    }

    fn get_config_dir(&self) -> PathBuf {
        self.path().join("config")
    }

    fn get_files_dir(&self) -> PathBuf {
        self.path().join("files")
    }

    fn assert_files<P>(&self, reference: &[P])
    where
        P: AsRef<Path>,
    {
        for path in reference {
            let child = self.tempdir.child(path);

            child.assert(predicate::path::exists());
        }
    }
}

fn rename_typical_input(env: &TestEnv) {
    let config_dir = env.get_config_dir();

    let mut cmd = Command::cargo_bin("tfmt").unwrap();

    let assert = cmd
        .arg("--config")
        .arg(config_dir)
        .arg("rename")
        .arg("typical_input")
        .arg("myname")
        .current_dir(env.tempdir.path())
        .assert();

    assert.success();
}

fn undo(env: &TestEnv) {
    let config_dir = env.get_config_dir();

    let mut cmd = Command::cargo_bin("tfmt").unwrap();

    let assert = cmd
        .arg("--config")
        .arg(config_dir)
        .arg("undo")
        .current_dir(env.tempdir.path())
        .assert();

    assert.success();
}

fn redo(env: &TestEnv) {
    let config_dir = env.get_config_dir();

    let mut cmd = Command::cargo_bin("tfmt").unwrap();

    let assert = cmd
        .arg("--config")
        .arg(config_dir)
        .arg("redo")
        .current_dir(env.tempdir.path())
        .assert();

    assert.success();
}

#[test]
fn test_rename_simple_input() -> Result<()> {
    let reference: [&str; 5] = [
        "MASTER BOOT RECORD/Dune.mp3",
        "MASTER BOOT RECORD/SET MIDI=SYNTH1 MAPG MODE1.mp3",
        "Amon Amarth/Under Siege.mp3",
        "Damjan Mravunac/Welcome To Heaven.ogg",
        "Nightwish/While Your Lips Are Still Red.mp3",
    ];

    test_runner(
        TestEnv::new,
        |_| Ok(()),
        |env| {
            let config_dir = env.get_config_dir();

            let mut cmd = Command::cargo_bin("tfmt").unwrap();

            let assert = cmd
                .arg("--config")
                .arg(config_dir)
                .arg("rename")
                .arg("simple_input")
                .current_dir(env.tempdir.path())
                .assert();

            assert.success();

            env.assert_files(&reference);

            Ok(())
        },
    )
}

#[test]
fn test_rename_typical_input() -> Result<()> {
    test_runner(
        TestEnv::new,
        |_| Ok(()),
        |env| {
            rename_typical_input(env);

            env.assert_files(&TYPICAL_INPUT_REFERENCE);

            Ok(())
        },
    )
}

#[test]
fn test_undo_typical_input() -> Result<()> {
    test_runner(
        TestEnv::new,
        |_| Ok(()),
        |env| {
            rename_typical_input(env);
            env.assert_files(&TYPICAL_INPUT_REFERENCE);

            undo(env);
            env.assert_files(&INITIAL_REFERENCE);

            Ok(())
        },
    )
}

#[test]
fn test_redo_typical_input() -> Result<()> {
    test_runner(
        TestEnv::new,
        |_| Ok(()),
        |env| {
            rename_typical_input(env);
            env.assert_files(&TYPICAL_INPUT_REFERENCE);

            undo(env);
            env.assert_files(&INITIAL_REFERENCE);

            redo(env);
            env.assert_files(&TYPICAL_INPUT_REFERENCE);

            Ok(())
        },
    )
}
