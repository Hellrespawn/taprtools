use crate::{Action, ActionGroup, DiskHandler, Result};
use log::{debug, info};
use std::fmt;
use std::path::Path;

/// History is responsible for saving and loading `ActionGroup`s
pub struct History {
    disk_handler: DiskHandler,
    current_group: ActionGroup,
    applied_groups: Vec<ActionGroup>,
    undone_groups: Vec<ActionGroup>,
}

impl fmt::Display for History {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "History file at {}", self.disk_handler.path().display())?;

        writeln!(f, "Applied actions ({}):", self.applied_groups.len())?;

        for group in &self.applied_groups {
            writeln!(f, "{group}")?;
        }

        writeln!(f, "Undone actions ({}):", self.undone_groups.len())?;

        for group in &self.undone_groups {
            writeln!(f, "{group}")?;
        }

        Ok(())
    }
}

impl History {
    /// Load or create history file at `path`
    pub fn load(path: &Path) -> Result<Self> {
        let disk_handler = DiskHandler::init(path);
        let (applied_groups, undone_groups) = disk_handler.read()?;

        info!("Loading history from {}", path.display());

        Ok(History {
            disk_handler,
            current_group: ActionGroup::new(),
            applied_groups,
            undone_groups,
        })
    }

    fn changed(&self) -> bool {
        self.current_group.changed()
            || self.applied_groups.iter().any(ActionGroup::changed)
            || self.undone_groups.iter().any(ActionGroup::changed)
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
        self.applied_groups = Vec::new();
        self.undone_groups = Vec::new();

        self.clear_on_disk()?;

        info!("History cleared.");

        Ok(())
    }

    /// Save history, if necessary
    pub fn save(&mut self) -> Result<()> {
        if !self.changed() {
            info!("Nothing was changed.");
            return Ok(());
        }

        if self.current_group.changed() {
            debug!("Current group was changed");
            let saved_group = std::mem::take(&mut self.current_group);

            self.applied_groups.push(saved_group);
        }

        self.save_to_disk()?;
        info!("Saved history to disk");

        Ok(())
    }

    /// Apply an action to the current `ActionGroup`.
    pub fn apply(&mut self, action: Action) -> Result<()> {
        self.current_group.apply(action)?;
        Ok(())
    }

    /// Rollback all changes in the current `ActionGroup`.
    pub fn rollback(&mut self) -> Result<()> {
        info!("Rolling back current group");
        let mut current_group = std::mem::take(&mut self.current_group);
        current_group.undo()?;
        Ok(())
    }

    /// Undo `n` amount of `ActionGroup`s. Returns amount actually undone
    pub fn undo(&mut self, amount: usize) -> Result<usize> {
        if amount == 0 {
            return Ok(0);
        }

        let mut i = 0;

        while let Some(mut group) = self.applied_groups.pop() {
            group.undo()?;
            self.undone_groups.push(group);

            i += 1;

            if i == amount {
                break;
            }
        }

        self.save()?;

        Ok(i)
    }

    /// Redo `n` amount of `ActionGroup`s. Returns amount actually redone
    pub fn redo(&mut self, amount: usize) -> Result<usize> {
        if amount == 0 {
            return Ok(0);
        }

        let mut i = 0;

        while let Some(mut group) = self.undone_groups.pop() {
            group.redo()?;
            self.applied_groups.push(group);

            i += 1;

            if i == amount {
                break;
            }
        }

        self.save()?;

        Ok(i)
    }
}
