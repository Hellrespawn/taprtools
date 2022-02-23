use crate::{ActionGroup, Result};
use std::collections::VecDeque;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(crate) struct DiskHandler {
    path: PathBuf,
}

impl DiskHandler {
    pub(crate) fn init<P>(path: P) -> DiskHandler
    where
        P: AsRef<Path>,
    {
        DiskHandler {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub(crate) fn clear(&self) -> Result<bool> {
        match std::fs::remove_file(&self.path) {
            Ok(_) => Ok(true),
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Ok(false)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub(crate) fn read(
        &self,
    ) -> Result<(VecDeque<ActionGroup>, VecDeque<ActionGroup>)> {
        match std::fs::read(&self.path) {
            Ok(file_contents) => {
                let (applied_actions, undone_actions): (
                    VecDeque<ActionGroup>,
                    VecDeque<ActionGroup>,
                ) = bincode::deserialize(&file_contents)?;

                Ok((applied_actions, undone_actions))
            }
            Err(err) => {
                if let ErrorKind::NotFound = err.kind() {
                    Ok((VecDeque::new(), VecDeque::new()))
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub(crate) fn write(
        &self,
        applied_actions: &VecDeque<ActionGroup>,
        undone_actions: &VecDeque<ActionGroup>,
    ) -> Result<()> {
        let serialized =
            bincode::serialize(&(applied_actions, undone_actions))?;
        std::fs::write(&self.path, serialized)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;
    use tempfile::{Builder, NamedTempFile};

    static PREFIX: &str = "rust-file-history-disk-";

    fn get_temporary_file() -> Result<NamedTempFile> {
        let file = Builder::new().prefix(PREFIX).tempfile()?;
        Ok(file)
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

    fn get_test_queue() -> VecDeque<ActionGroup> {
        let mut queue = VecDeque::new();
        queue.push_front(get_test_group());
        queue.push_front(get_test_group());
        queue.push_front(get_test_group());
        queue.push_front(get_test_group());
        queue
    }

    fn write_read_compare_test_data(disk_handler: &DiskHandler) -> Result<()> {
        let applied_actions_in = get_test_queue();
        let undone_actions_in = get_test_queue();

        disk_handler.write(&applied_actions_in, &undone_actions_in)?;

        let (applied_actions_out, undone_actions_out) = disk_handler.read()?;

        assert_eq!(applied_actions_in, applied_actions_out);
        assert_eq!(undone_actions_in, undone_actions_out);

        Ok(())
    }

    #[test]
    fn test_write_and_read() -> Result<()> {
        let file = get_temporary_file()?;
        let disk_handler = DiskHandler::init(&file.path());

        write_read_compare_test_data(&disk_handler)?;

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<()> {
        let file = get_temporary_file()?;
        let disk_handler = DiskHandler::init(&file.path());

        assert!(file.path().is_file());

        assert_eq!(disk_handler.clear()?, true);

        // These two indicate the same thing.
        assert_eq!(disk_handler.clear()?, false);
        assert!(!file.path().exists());

        Ok(())
    }

    #[test]
    fn test_write_and_read_from_clear() -> Result<()> {
        let file = get_temporary_file()?;
        let disk_handler = DiskHandler::init(&file.path());

        assert_eq!(disk_handler.clear()?, true);
        assert_eq!(disk_handler.clear()?, false);

        write_read_compare_test_data(&disk_handler)?;

        assert_eq!(disk_handler.clear()?, true);
        assert_eq!(disk_handler.clear()?, false);

        Ok(())
    }
}
