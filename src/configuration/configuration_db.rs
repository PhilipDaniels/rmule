use crate::file;
use anyhow::Result;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::cell::{Ref, RefCell};
use std::path::{Path, PathBuf};

use super::migrations;

pub struct ConfigurationDb {
    pub config_dir: PathBuf,
    config_db_filename: PathBuf,
    conn: RefCell<Connection>,
}

impl ConfigurationDb {
    /// Loads new configuration from disk. If the configuration database does
    /// not exist then a new one is created. If the database requires an upgrade,
    /// then that is applied first before the configuration is returned.
    ///
    /// This function should be called at startup, once it has run then
    /// the database is accessed by individual methods as needed.
    pub fn open(config_dir: &Path) -> Result<Self> {
        let filename = Self::config_db_filename(config_dir);

        eprintln!(
            "Attempting to open configuration database {}",
            filename.display()
        );

        // This will create an empty SQLite db if needed.
        let conn = Connection::open(&filename)?;

        migrations::apply_database_migrations(&conn)?;

        eprintln!("Opened configuration database {}", filename.display());

        let cfg = Self {
            config_dir: config_dir.to_owned(),
            config_db_filename: filename,
            conn: RefCell::new(conn),
        };

        Ok(cfg)
    }

    /// Backs up the current configuration database.
    /// Deletes any out of date backups.
    pub fn backup(config_dir: &Path) -> Result<()> {
        let filename = Self::config_db_filename(config_dir);
        if filename.try_exists()? {
            let backup_config_file = file::make_backup_filename(&filename);
            std::fs::copy(filename, backup_config_file)?;
            let num_deleted = file::delete_backups(config_dir, "rmule.sqlite", 10)?;
            eprintln!("Deleted {} backups of rmule.sqlite", num_deleted);
        }

        Ok(())
    }

    /// Deletes the current configuration database.
    pub fn delete(config_dir: &Path) -> Result<()> {
        let filename = Self::config_db_filename(config_dir);
        std::fs::remove_file(filename)?;
        Ok(())
    }

    fn config_db_filename<P: Into<PathBuf>>(config_dir: P) -> PathBuf {
        let mut p = config_dir.into();
        p.push("rmule.sqlite");
        p
    }

    /// Get the connection. Most ops can be performed on
    /// a shared connection.
    pub fn conn(&self) -> Ref<Connection> {
        self.conn.borrow()
    }

    /// Executes a transaction on this database.
    /// See [https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html]
    pub fn execute_in_transaction<T, F>(
        &self,
        behaviour: TransactionBehavior,
        block: F,
    ) -> Result<T>
    where
        F: FnOnce(Transaction) -> Result<T>,
    {
        let mut conn = self.conn.borrow_mut();
        let txn = Transaction::new(&mut conn, behaviour)?;
        let result = block(txn)?;
        Ok(result)
    }
}
