use crate::{Action, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct ActionGroup(VecDeque<Action>);

impl ActionGroup {
    pub(crate) fn new() -> Self {
        ActionGroup(VecDeque::new())
    }

    pub(crate) fn changed(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn apply(&mut self, action: Action) -> Result<()> {
        action.apply()?;
        self.0.push_front(action);
        Ok(())
    }

    pub(crate) fn undo(&mut self) -> Result<()> {
        for action in &self.0 {
            action.undo()?;
        }

        Ok(())
    }

    pub(crate) fn redo(&mut self) -> Result<()> {
        for action in &self.0 {
            action.apply()?;
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn insert(&mut self, action: Action) {
        self.0.push_front(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::{Builder, TempDir};

    static PREFIX: &str = "rust-file-history-actiongroup-";

    fn get_temporary_dir() -> Result<TempDir> {
        let dir = Builder::new().prefix(PREFIX).tempdir()?;
        Ok(dir)
    }

    fn get_test_group() -> ActionGroup {
        let mut action_group = ActionGroup::new();

        action_group
            .insert(Action::MakeDir(PathBuf::from("/file/test/create")));
        action_group
            .insert(Action::RemoveDir(PathBuf::from("/file/test/remove")));
        action_group.insert(Action::Move {
            source: PathBuf::from("/file/test/source"),
            target: PathBuf::from("/file/test/target"),
        });

        action_group
    }

    fn test_action_apply(action: &Action) -> Result<()> {
        match action {
            Action::MakeDir(_) => Ok(()),
            Action::RemoveDir(_) => Ok(()),
            Action::Move {
                source: _,
                target: _,
            } => Ok(()),
        }
    }

    fn test_action_undo(action: &Action) -> Result<()> {
        Ok(())
    }



    #[test]
    fn test_actiongroup() -> Result<()> {
        let dir = get_temporary_dir()?;
        let group = get_test_group();

        for action in group.0 {}

        Ok(())
    }
}
