use crate::{Action, ActionGroup, DiskHandler, Result};
use std::collections::VecDeque;
use std::path::Path;

/// History is responsible for saving and loading ActionGroups
pub struct History {
    disk_handler: DiskHandler,
    current_group: ActionGroup,
    applied_groups: VecDeque<ActionGroup>,
    undone_groups: VecDeque<ActionGroup>,
    changed: bool,
}

impl History {
    /// Load or create history file at `path`
    pub fn load(path: &Path) -> Result<Self> {
        let disk_handler = DiskHandler::init(path);
        let (applied_groups, undone_groups) = disk_handler.read()?;

        Ok(History {
            disk_handler,
            current_group: ActionGroup::new(),
            applied_groups,
            undone_groups,
            changed: false,
        })
    }

    fn save_to_disk(&self) -> Result<()> {
        self.disk_handler
            .write(&self.applied_groups, &self.undone_groups)
    }

    fn clear_on_disk(&self) -> Result<()> {
        self.disk_handler.clear()?;
        Ok(())
    }

    /// Clears history
    pub fn clear(&mut self) -> Result<()> {
        self.current_group = ActionGroup::new();
        self.applied_groups = VecDeque::new();
        self.undone_groups = VecDeque::new();
        self.changed = false;

        self.clear_on_disk()?;

        Ok(())
    }

    /// Save history, if necessary
    pub fn save(&mut self) -> Result<()> {
        if !self.changed {
            // Do nothing
            return Ok(());
        }

        let saved_group = std::mem::take(&mut self.current_group);

        self.applied_groups.push_front(saved_group);
        self.save_to_disk()?;

        Ok(())
    }

    /// Apply an action to the current [ActionGroup].
    pub fn apply(&mut self, action: Action) -> Result<()> {
        self.current_group.apply(action)?;
        self.changed = true;
        Ok(())
    }

    /// Rollback all changes in the current [ActionGroup].
    pub fn rollback(&mut self) -> Result<()> {
        let mut current_group = std::mem::take(&mut self.current_group);
        current_group.undo()?;
        self.changed = false;
        Ok(())
    }

    /// Undo `n` amount of [ActionGroup]s. Returns amount actually undone
    pub fn undo(&mut self, amount: usize) -> Result<usize> {
        for i in 0..amount {
            if let Some(mut group) = self.applied_groups.pop_front() {
                group.undo()?;
                self.undone_groups.push_front(group);
                self.changed = true;
            } else {
                self.save_to_disk()?;
                return Ok(i);
            }
        }

        self.save_to_disk()?;
        Ok(amount)
    }

    /// Redo `n` amount of [ActionGroup]s. Returns amount actually redone
    pub fn redo(&mut self, amount: usize) -> Result<usize> {
        for i in 0..amount {
            if let Some(mut group) = self.undone_groups.pop_front() {
                group.redo()?;
                self.applied_groups.push_front(group);
                self.changed = true;
            } else {
                self.save_to_disk()?;
                return Ok(i);
            }
        }

        self.save_to_disk()?;
        Ok(amount)
    }
}
