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
