use anyhow::Result;
use std::path::PathBuf;

pub(crate) enum ActionType {
    Undo,
    Redo,
}

pub(crate) fn main(
    action_type: ActionType,
    preview: bool,
    times: usize,
    config: PathBuf,
) -> Result<()> {
    todo!()
}
