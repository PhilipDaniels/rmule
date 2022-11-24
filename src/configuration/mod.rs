use anyhow::Result;
use rusqlite::Connection;
use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::{file, times};

mod migrations;
mod sqlite_extensions;

pub struct ConfigurationDb {
    pub config_dir: PathBuf,
    config_db_filename: PathBuf,
    conn: Connection,
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

        // This will create an empty SQLite db if needed.
        let conn = Connection::open(&filename)?;

        migrations::apply_database_migrations(&conn)?;

        let cfg = Self {
            config_dir: config_dir.to_owned(),
            config_db_filename: filename,
            conn,
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
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // pub fn conn_mut(&mut self) -> &mut Connection {
    //     &mut self.conn
    // }
}

#[derive(Debug)]
pub struct Settings {
    pub downloaded_directory: PathBuf,
    pub nick_name: String,
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let mut settings = db.conn.query_row("SELECT * FROM settings", [], |row| {
            Ok(Self {
                downloaded_directory: row.get::<_, String>("downloaded_directory")?.into(),
                nick_name: row.get("nick_name")?,
            })
        })?;

        if settings.make_paths_absolute(&db.config_dir) {
            settings.save(db)?;
        }

        Ok(settings)
    }

    /// Saves the settings to the database.
    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn.execute(
            r#"UPDATE settings SET
                downloaded_directory = ?1,
                nick_name = ?2,
                updated = ?3
            "#,
            [
                &self.downloaded_directory.to_string_lossy().into_owned(),
                &self.nick_name,
                &times::now_to_sql()
            ],
        )?;

        Ok(())
    }

    /// Ensures that all the paths on any settings object are all absolute paths.
    fn make_paths_absolute(&mut self, dir: &Path) -> bool {
        let mut did_change = false;

        match file::make_absolute(&self.downloaded_directory, dir) {
            Cow::Borrowed(_) => {}
            Cow::Owned(p) => {
                self.downloaded_directory = p;
                did_change = true;
            }
        }

        did_change
    }
}
