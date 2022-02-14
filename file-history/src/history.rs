use crate::{Action, ActionGroup, Result};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;

/// History is responsible for saving and loading ActionGroups
pub struct History {
    current_group: ActionGroup,
    applied_groups: VecDeque<ActionGroup>,
    undone_groups: VecDeque<ActionGroup>,

    path: PathBuf,
}

impl History {
    /// Load or create history file at `path`
    pub fn init(path: &Path) -> Result<Self> {
        let (applied_groups, undone_groups) =
            History::read_from_database(path)?;

        Ok(History {
            current_group: ActionGroup::new(),
            applied_groups,
            undone_groups,
            path: path.to_path_buf(),
        })
    }

    fn read_from_database(
        _path: &Path,
    ) -> Result<(VecDeque<ActionGroup>, VecDeque<ActionGroup>)> {
        Ok((VecDeque::new(), VecDeque::new()))
    }

    fn write_to_database(&self) -> Result<()> {
        Ok(())
    }

    /// Save history, if necessary
    pub fn save(&mut self) -> Result<()> {
        if self.current_group.is_empty() {
            // Do nothing
            return Ok(());
        }

        let saved_group = std::mem::take(&mut self.current_group);

        self.applied_groups.push_front(saved_group);
        self.write_to_database()?;

        Ok(())
    }

    /// Apply an action to the current [ActionGroup].
    pub fn apply(&mut self, action: Action) -> Result<()> {
        self.current_group.apply(action)
    }

    /// Rollback all changes in the current [ActionGroup].
    pub fn rollback(&mut self) -> Result<()> {
        let mut current_group = std::mem::take(&mut self.current_group);
        current_group.undo()
    }

    /// Undo `n` amount of [ActionGroup]s. Returns amount actually undone
    pub fn undo(&mut self, amount: usize) -> Result<usize> {
        for i in 0..amount {
            if let Some(mut group) = self.applied_groups.pop_front() {
                group.undo()?;
                self.undone_groups.push_front(group);
            } else {
                return Ok(i)
            }
        }

        Ok(amount)
    }

    /// Redo `n` amount of [ActionGroup]s. Returns amount actually redone
    pub fn redo(&mut self, amount: usize) -> Result<usize> {
        for i in 0..amount {
            if let Some(mut group) = self.undone_groups.pop_front() {
                group.redo()?;
                self.applied_groups.push_front(group);
            } else {
                return Ok(i)
            }
        }

        Ok(amount)
    }
}
