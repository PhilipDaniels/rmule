use std::path::PathBuf;

use super::sqlite_extensions::DatabasePathBuf;
use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::{params, TransactionBehavior};

/// The rmule equivalent of the "temp directory" setting from emule.
/// rmule supports multiple temp directories, which can help with
/// spreading disk IO across multiple devices, in case you don't have a
/// RAID array. They must be unique though, there is no point in having
/// multiple temp directories pointing to the same physical directory.
pub struct TempDirectoryList {
    directories: Vec<DatabasePathBuf>,
}

impl TempDirectoryList {
    /// Load all download directories from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT directory FROM temp_directory")?;

        let mut directories: Vec<DatabasePathBuf> = stmt
            .query_map([], |row| Ok(row.get("directory")?))?
            .flatten()
            .collect();

        let mut made_abs = false;
        for dir in &mut directories {
            if dir.make_absolute(&db.config_dir) {
                eprintln!(
                    "Made temp_directory absolute, is now {}",
                    dir.to_string_lossy()
                );
                made_abs = true;
            }
        }

        // nasty
        drop(stmt);
        drop(conn);

        let tdl = Self { directories };

        if made_abs {
            tdl.save(db)?;
        }

        eprintln!("Loaded {} rows from temp_directory", tdl.directories.len());

        Ok(tdl)
    }

    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.execute_in_transaction(TransactionBehavior::Deferred, |txn| {
            txn.execute("DELETE FROM temp_directory", [])?;

            let mut stmt = txn.prepare("INSERT INTO temp_directory(directory) VALUES (?1)")?;

            for dir in &self.directories {
                stmt.execute(params![dir])?;
            }

            stmt.finalize()?;

            txn.commit()?;
            eprintln!("Saved {} rows to temp_directory", self.directories.len());
            Ok(())
        })?;

        Ok(())
    }

    /// Adds a new directory to the list but only if it is not already in
    /// the list.
    pub fn add<P: Into<PathBuf>>(&mut self, dir: P) {
        let dir = dir.into().into();
        self.directories.push(dir);
        self.directories.dedup();
    }
}
