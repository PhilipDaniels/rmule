use super::sqlite_extensions::DatabasePathBuf;
use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::TransactionBehavior;

/// The rmule equivalent of the "temp directory" setting from emule.
/// rmule supports multiple temp directories, which can help with
/// spreading disk IO across multiple devices, in case you don't have a
/// RAID array. They must be unique though, there is no point in having
/// multiple temp directories pointing to the same physical directory.
pub struct TempDirectoryList {
    directories: Vec<TempDirectory>,
}

pub struct TempDirectory {
    /// The directory in which to store the temporary download database.
    directory: DatabasePathBuf,
}

impl TempDirectoryList {
    /// Load all download directories from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT directory FROM temp_directory")?;

        let temp_dir_iter = stmt.query_map([], |row| {
            Ok(TempDirectory {
                directory: row.get("directory")?,
            })
        })?;

        Ok(Self {
            directories: temp_dir_iter.flatten().collect(),
        })
    }

    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.execute_in_transaction(TransactionBehavior::Deferred, |txn| {
            txn.execute("DELETE temp_directory", [])?;

            Ok(())
        })?;

        Ok(())
    }
}
