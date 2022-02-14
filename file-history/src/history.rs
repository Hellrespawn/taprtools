use crate::{Action, ActionGroup, Database, Result};
use std::collections::VecDeque;
use std::path::Path;

/// History is responsible for saving and loading ActionGroups
pub struct History {
    database: Database,
    current_group: ActionGroup,
    applied_groups: VecDeque<ActionGroup>,
    undone_groups: VecDeque<ActionGroup>,
}

impl History {
    /// Load or create history file at `path`
    pub fn init(path: &Path) -> Result<Self> {
        let database = Database::connect(path)?;
        let (applied_groups, undone_groups) = database.read()?;

        Ok(History {
            database,
            current_group: ActionGroup::new(),
            applied_groups,
            undone_groups,
        })
    }

    fn write_to_database(&self) -> Result<()> {
        self.database.write()
    }

    fn clear_database(&self) -> Result<()> {
        self.database.clear()
    }

    /// Clears history
    pub fn clear(&mut self) -> Result<()> {
        self.current_group = ActionGroup::new();
        self.applied_groups = VecDeque::new();
        self.undone_groups = VecDeque::new();

        self.clear_database()?;

        Ok(())
    }

    /// Save history, if necessary
    pub fn save(&mut self) -> Result<()> {
        if self.current_group.changed() {
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
                return Ok(i);
            }
        }

        self.write_to_database()?;
        Ok(amount)
    }

    /// Redo `n` amount of [ActionGroup]s. Returns amount actually redone
    pub fn redo(&mut self, amount: usize) -> Result<usize> {
        for i in 0..amount {
            if let Some(mut group) = self.undone_groups.pop_front() {
                group.redo()?;
                self.applied_groups.push_front(group);
            } else {
                return Ok(i);
            }
        }

        self.write_to_database()?;
        Ok(amount)
    }
}
