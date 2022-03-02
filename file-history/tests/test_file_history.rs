use anyhow::Result;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use file_history::{Action, History};
use predicates::prelude::*;

// TODO Write undo/redo tests

#[test]
fn test_new_history() -> Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("test.histfile");

    History::load(&path)?;

    path.assert(predicate::path::missing());

    Ok(())
}

#[test]
fn test_save_unchanged_history() -> Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("test.histfile");

    let mut history = History::load(&path)?;

    path.assert(predicate::path::missing());

    history.save()?;

    path.assert(predicate::path::missing());

    Ok(())
}

// FIXME Fix this test.
#[test]
fn test_save_after_rollback() -> Result<()> {
    let dir = TempDir::new()?;

    let history_path = dir.child("test.histfile");
    let mut history = History::load(&history_path)?;

    history.save()?;

    history_path.assert(predicate::path::missing());

    let source = dir.child("source");
    source.assert(predicate::path::missing());

    source.touch().unwrap();

    source.assert(predicate::path::is_file());

    let target = source.child("target");
    target.assert(predicate::path::missing());

    let action = Action::Move {
        source: source.to_path_buf(),
        target: target.to_path_buf(),
    };

    history.apply(action)?;

    source.assert(predicate::path::missing());
    target.assert(predicate::path::is_file());

    history.rollback()?;

    source.assert(predicate::path::is_file());
    target.assert(predicate::path::missing());

    history.save()?;

    history_path.assert(predicate::path::missing());
    Ok(())
}
