use crate::util::calculate_hash;
use crate::{Action, ActionType, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub struct ActionCount {
    pub mv: u64,
    pub mkdir: u64,
    pub rmdir: u64,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub(crate) struct ActionGroup {
    actions: Vec<Action>,
    hash: u64,
}

impl fmt::Display for ActionGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut move_count = 0;
        let mut mkdir_count = 0;
        let mut rmdir_count = 0;

        for action in &self.actions {
            match action.action_type() {
                ActionType::Mv { .. } => move_count += 1,
                ActionType::MkDir(_) => mkdir_count += 1,
                ActionType::RmDir(_) => rmdir_count += 1,
            }
        }

        writeln!(
            f,
            "mv: {}, mkdir: {}, rmdir: {}",
            move_count, mkdir_count, rmdir_count
        )?;

        for action in &self.actions {
            writeln!(f, "{}", action)?;
        }

        Ok(())
    }
}

impl PartialEq for ActionGroup {
    fn eq(&self, other: &Self) -> bool {
        self.actions == other.actions
    }
}

impl ActionGroup {
    pub(crate) fn new() -> Self {
        let actions = Vec::new();
        let hash = calculate_hash(&actions);
        ActionGroup { actions, hash }
    }

    pub(crate) fn update_hash(&mut self) {
        self.hash = calculate_hash(&self.actions);
    }

    pub(crate) fn to_action_count(&self) -> ActionCount {
        let mut action_count = ActionCount {
            mv: 0,
            mkdir: 0,
            rmdir: 0,
        };

        for action in &self.actions {
            match action.action_type() {
                ActionType::Mv { .. } => action_count.mv += 1,
                ActionType::MkDir(_) => action_count.mkdir += 1,
                ActionType::RmDir(_) => action_count.rmdir += 1,
            }
        }

        action_count
    }

    // pub(crate) fn to_string_short(&self) -> String {
    //     let string = self.to_string();
    //     let lines: Vec<&str> = string.lines().collect();
    //     format!("{}{}{}", lines[0], lines[1], lines[self.0.len() + 2])
    // }

    pub(crate) fn changed(&self) -> bool {
        self.hash != calculate_hash(&self.actions)
    }

    pub(crate) fn apply(&mut self, mut action: Action) -> Result<()> {
        action.apply()?;
        self.actions.push(action);
        Ok(())
    }

    pub(crate) fn undo(&mut self) -> Result<()> {
        // Undo happens in reverse order
        for action in self.actions.iter_mut().rev() {
            action.undo()?;
        }

        Ok(())
    }

    pub(crate) fn redo(&mut self) -> Result<()> {
        // Redo happens in original order
        for action in &mut self.actions {
            action.apply()?;
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push(&mut self, action: Action) {
        self.actions.push(action);
    }
}
