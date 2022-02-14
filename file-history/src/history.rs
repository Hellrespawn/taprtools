use crate::Result;
use std::path::{Path, PathBuf};

// Turn vecs into VecDeque

// History is responsible for saving and loading ActionGroups
pub struct History {
    current_group: ActionGroup,
    applied_groups: Vec<ActionGroup>,
    undone_groups: Vec<ActionGroup>,

    path: PathBuf,
}

impl History {
    pub fn load(path: &Path) -> Result<Self> {
        let applied_groups = History::read_from_database(path)?;
        Ok(History {
            current_group: ActionGroup::new(),
            applied_groups,
            undone_groups: Vec::new(),
            path: path.to_path_buf(),
        })
    }

    /// Read ActionGroups from SQLite database
    fn read_from_database(path: &Path) -> Result<Vec<ActionGroup>> {
        Ok(Vec::new())
    }

    /// Read ActionGroups from SQLite database
    fn write_to_database(&self) -> Result<()> {
        Ok(())
    }

    pub fn save(&mut self) -> Result<()> {
        if self.current_group.is_empty() {
            // Do nothing
            return Ok(());
        }

        let saved_group = std::mem::take(&mut self.current_group);

        self.applied_groups.insert(0, saved_group);
        self.write_to_database()?;

        Ok(())
    }

    pub fn apply(&mut self, action: Action) -> Result<()> {
        self.current_group.apply(action)
    }

    pub fn rollback(&mut self) -> Result<()> {
        let mut current_group = std::mem::take(&mut self.current_group);
        current_group.undo()
    }

    pub fn undo(&mut self, amount: usize) -> Result<()> {
        // FIXME use VecDequeue to pop and insert better.
        for _ in 0..amount {
            if let Some(mut group) = self.applied_groups.pop() {
                group.undo()?;
                self.undone_groups.insert(0, group);
            }
        }

        Ok(())
    }
}

// ActionGroup is responsible for collecting all Actions belonging to a single group. Also rolling back?
#[derive(Default)]
pub struct ActionGroup(Vec<Action>);

impl ActionGroup {
    pub fn new() -> Self {
        ActionGroup(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn apply(&mut self, action: Action) -> Result<()> {
        action.apply()?;
        self.0.insert(0, action);
        Ok(())
    }

    pub fn undo(&mut self) -> Result<()> {
        for action in self.0.iter() {
            action.undo()?;
        }

        Ok(())
    }
}

/// Action is responsible for doing and undoing filesystem operations
#[derive(Debug)]
pub enum Action {
    Move { source: PathBuf, target: PathBuf },
    CreateDir(PathBuf),
    RemoveDir(PathBuf),
}

impl Action {
    pub fn apply(&self) -> Result<()> {
        Ok(())
    }

    pub fn undo(&self) -> Result<()> {
        Ok(())
    }
}
