use anyhow::Result;
use assert_fs::TempDir;
use file_history::History;

// TODO Write undo/redo tests

const FILE_NAME: &str = "test.histfile";

#[test]
fn test_new_history_doesnt_create_file() -> Result<()> {
    let dir = TempDir::new()?;

    let history = History::load(&dir.path(), FILE_NAME)?;

    assert!(!history.path().exists());

    Ok(())
}

#[test]
fn test_unchanged_history_doesnt_save() -> Result<()> {
    let dir = TempDir::new()?;

    let mut history = History::load(&dir.path(), FILE_NAME)?;

    assert!(matches!(history.save(), Ok(false)));

    assert!(!history.path().exists());

    Ok(())
}
