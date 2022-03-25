use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use file_history::{Action, History};
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

#[test]
fn test_save_after_rollback() -> Result<()> {
    // Set up directory
    let dir = TempDir::new()?;

    // Set up history
    let history_path = dir.child("test.histfile");
    let mut history = History::load(&history_path)?;

    // Set up source file
    let source = dir.child("source");
    source.assert(predicate::path::missing());
    source.touch().unwrap();
    source.assert(predicate::path::is_file());

    // Set up target file
    let target = dir.child("target");
    target.assert(predicate::path::missing());

    // Perform action
    let action = Action::Move {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
    };

    history.apply(action)?;

    // Assert file was moved
    source.assert(predicate::path::missing());
    target.assert(predicate::path::is_file());

    // Rollback and assert files are back
    history.rollback()?;

    source.assert(predicate::path::is_file());
    target.assert(predicate::path::missing());

    // Save and assert no history file was created.
    history.save()?;

    history_path.assert(predicate::path::missing());
    Ok(())
}
