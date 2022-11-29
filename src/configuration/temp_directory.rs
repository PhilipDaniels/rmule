use std::path::{Path, PathBuf};

use super::sqlite_extensions::DatabasePathBuf;
use super::ConfigurationDb;
use anyhow::{bail, Result};
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

        eprintln!("Loaded {} rows from temp_directory", directories.len());

        Ok(Self { directories })
    }

    /// Makes any paths found in the directories list into absolute ones.
    /// This is not possible for paths added via the 'add' method because
    /// it guards against relative paths, but for initial data inserted into
    /// the database at migration time it can happen.
    ///
    /// We have a post-condition that any paths returned from the configuration
    /// db to the wider program will always be absolute paths, and this
    /// method helps to enforce that.
    ///
    /// Returns the number of directories changed.
    pub fn make_absolute(&mut self, within_dir: &Path) -> usize {
        let mut num_made_abs = 0;

        for dir in &mut self.directories {
            if dir.make_absolute(within_dir) {
                eprintln!(
                    "Made temp_directory absolute, is now {}",
                    dir.to_string_lossy()
                );
                num_made_abs += 1;
            }
        }

        num_made_abs
    }

    /// Saves the temp directory list to the database. The entire list is
    /// saved in one transaction.
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
    /// the list. The directory must be absolute - an error is returned
    /// if it isn't.
    ///
    /// Returns 1 if the directory is added, 0 otherwise.
    ///
    /// # Remarks
    /// You might think that the function could return a bool, and you
    /// would be right, but that makes it very easy to introduce bugs
    /// in the calling code via boolean shortcircuiting evaluation.
    /// e.g.
    ///     let added = added || temp_dirs.add("foo")?;
    /// will never add 'foo' if added is already true.
    pub fn add<P: Into<PathBuf>>(&mut self, dir: P) -> Result<usize> {
        let dir: PathBuf = dir.into();
        if !dir.is_absolute() {
            bail!("Directory {} is not absolute", dir.to_string_lossy());
        }

        let dir = dir.into();
        if !self.directories.contains(&dir) {
            eprintln!("Adding temp_directory {}", dir.to_string_lossy());
            self.directories.push(dir);
            self.directories.sort();
            Ok(1)
        } else {
            Ok(0)
        }
    }
}
