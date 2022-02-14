use crate::{ActionGroup, Result};
use rusqlite::Connection;
use std::collections::VecDeque;
use std::path::Path;

// Load database on init
// Start transaction
// Do stuff
// Commit on save

pub(crate) struct Database {
    connection: Connection,
}

impl Database {
    pub(crate) fn connect<P>(path: P) -> Result<Database>
    where
        P: AsRef<Path>,
    {
        let connection = Connection::open(&path)?;

        // Enable foreign key constraints.
        connection.pragma_update(None, "foreign_keys", "ON")?;

        let database = Database { connection };
        database.create_tables()?;

        Ok(database)
    }

    pub(crate) fn clear(&self) -> Result<()> {
        self.drop_tables()
    }

    pub(crate) fn read(
        &self,
    ) -> Result<(VecDeque<ActionGroup>, VecDeque<ActionGroup>)> {
        Ok((VecDeque::new(), VecDeque::new()))
    }

    pub(crate) fn write(&self) -> Result<()> {
        Ok(())
    }

    fn create_tables(&self) -> Result<()> {
        self.create_table_action()?;
        self.create_table_actiongroup()?;
        Ok(())
    }

    fn drop_tables(&self) -> Result<()> {
        let tables = ["action", "actiongroup"];
        for table in tables {
            self.connection
                .execute(&format!("DROP TABLE IF EXISTS {table}"), [])?;
        }
        Ok(())
    }

    fn create_table_action(&self) -> Result<()> {
        let sql = "
        CREATE TABLE IF NOT EXISTS `action` (
            `id`	INTEGER NOT NULL,
            `type`	TEXT NOT NULL CHECK(`type` IN ('Move', 'MakeDir', 'RemoveDir')),
            `source`	TEXT NOT NULL,
            `target`	TEXT,
            `actiongroup_id`	INTEGER NOT NULL,
            FOREIGN KEY(`actiongroup_id`) REFERENCES `actiongroup`(`id`) ON DELETE CASCADE ON UPDATE CASCADE,
            PRIMARY KEY(`id` AUTOINCREMENT)
        )";
        self.connection.execute(sql, [])?;
        Ok(())
    }

    fn create_table_actiongroup(&self) -> Result<()> {
        let sql = "
        CREATE TABLE IF NOT EXISTS `actiongroup` (
            `id`	INTEGER NOT NULL,
            `date`	INTEGER NOT NULL,
            `undone`	BOOLEAN NOT NULL DEFAULT 'FALSE',
            PRIMARY KEY(`id` AUTOINCREMENT)
        )";
        self.connection.execute(sql, [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use tempfile::{Builder, TempDir};

    static INDEX: Lazy<Mutex<u64>> = Lazy::new(|| Mutex::new(0));

    fn get_next_index() -> u64 {
        let mut index = INDEX.lock().expect("Unable to lock Mutex!");
        *index += 1;
        *index - 1
    }

    fn get_testing_database() -> Result<(TempDir, Database)> {
        let index = get_next_index();
        let dir = Builder::new().prefix("rust-file-history-").tempdir()?;

        let path = dir.path().join(format!("test{index}.sqlite"));

        dbg!(&path);

        Ok((dir, Database::connect(path)?))
    }

    #[test]
    fn test_create_tables() -> Result<()> {
        let (dir, _) = get_testing_database()?;
        dir.close()?;
        Ok(())
    }
}
