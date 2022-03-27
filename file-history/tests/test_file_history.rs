use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use file_history::History;
use predicates::prelude::*;

// TODO Write undo/redo tests

#[test]
fn test_new_history_doesnt_create_file() -> Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("test.histfile");

    History::load(&path)?;

    path.assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_unchanged_history_doesnt_save() -> Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("test.histfile");

    let mut history = History::load(&path)?;

    assert!(matches!(history.save(), Ok(false)));
    path.assert(predicate::path::missing());

    Ok(())
}
