use file_history::{Action, History, Result};
use std::path::Path;
use tempfile::{Builder, NamedTempFile, TempDir};

// TODO Write undo/redo tests

static PREFIX: &str = "rust-file-history-history-";

fn get_temporary_dir() -> Result<TempDir> {
    let dir = Builder::new().prefix(PREFIX).tempdir()?;
    Ok(dir)
}

fn get_temporary_file(path: &Path) -> Result<NamedTempFile> {
    let tempfile = NamedTempFile::new_in(path)?;
    Ok(tempfile)
}

#[test]
fn test_new_history() -> Result<()> {
    let dir = get_temporary_dir()?;
    let path = dir.path().join("test.histfile");

    History::load(&path)?;

    assert!(!path.is_file());

    Ok(())
}

#[test]
fn test_save_unchanged_history() -> Result<()> {
    let dir = get_temporary_dir()?;
    let path = dir.path().join("test.histfile");

    let mut history = History::load(&path)?;

    assert!(!path.is_file());

    history.save()?;

    assert!(!path.is_file());

    Ok(())
}

#[test]
fn test_save_after_rollback() -> Result<()> {
    let dir = get_temporary_dir()?;

    let file = get_temporary_file(dir.path())?;
    assert!(file.path().is_file());

    let history_path = dir.path().join("test.histfile");
    let mut history = History::load(&history_path)?;
    assert!(!history_path.is_file());

    history.save()?;

    assert!(!history_path.is_file());

    let target = file.path().with_file_name("move_test");

    let action = Action::Move {
        source: file.path().to_path_buf(),
        target: target.to_path_buf(),
    };

    history.apply(action)?;

    assert!(!file.path().is_file());
    assert!(target.is_file());

    history.rollback()?;

    assert!(file.path().is_file());
    assert!(!target.is_file());

    history.save()?;

    assert!(!history_path.is_file());
    Ok(())
}
