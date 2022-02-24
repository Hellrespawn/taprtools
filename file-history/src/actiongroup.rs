use crate::{Action, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct ActionGroup {
    actions: Vec<Action>,
    #[serde(skip)] // Default for bool is false, so this just works.
    changed: bool,
}

impl fmt::Display for ActionGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut move_count = 0;
        let mut mkdir_count = 0;
        let mut rmdir_count = 0;

        for action in &self.actions {
            match action {
                Action::Move { .. } => move_count += 1,
                Action::MakeDir(_) => mkdir_count += 1,
                Action::RemoveDir(_) => rmdir_count += 1,
            }
        }

        writeln!(
            f,
            "mv: {move_count}, mkdir: {mkdir_count}, rmdir: {rmdir_count}"
        )?;

        for action in &self.actions {
            writeln!(f, "{action}")?;
        }

        Ok(())
    }
}

impl ActionGroup {
    pub(crate) fn new() -> Self {
        ActionGroup {
            actions: Vec::new(),
            changed: false,
        }
    }

    // pub(crate) fn to_string_short(&self) -> String {
    //     let string = self.to_string();
    //     let lines: Vec<&str> = string.lines().collect();
    //     format!("{}{}{}", lines[0], lines[1], lines[self.0.len() + 2])
    // }

    pub(crate) fn changed(&self) -> bool {
        self.changed
    }

    pub(crate) fn apply(&mut self, action: Action) -> Result<()> {
        action.apply()?;
        self.actions.push(action);
        self.changed = true;
        Ok(())
    }

    pub(crate) fn undo(&mut self) -> Result<()> {
        // Undo happens in reverse order
        for action in self.actions.iter().rev() {
            action.undo()?;
        }
        self.changed = true;

        Ok(())
    }

    pub(crate) fn redo(&mut self) -> Result<()> {
        // Redo happens in original order
        for action in &self.actions {
            action.apply()?;
        }
        self.changed = true;

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn push(&mut self, action: Action) {
        self.actions.push(action);
        self.changed = true;
    }
}
