use std::path::Path;

use super::sqlite_extensions::{DatabasePathBuf, DatabaseTime};
use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::params;
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub nick_name: String,
    /// When a download is started it is written to
    /// 1. The directory set for the download itself, if set.
    /// 2. This directory, if set.
    /// 3. The default default_downloads_directory on the Settings,
    /// which is always set.
    pub default_downloads_directory: DatabasePathBuf,
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let settings = db.conn().query_row("SELECT * FROM settings", [], |row| {
            Ok(Self {
                nick_name: row.get("nick_name")?,
                default_downloads_directory: row.get("default_downloads_directory")?,
            })
        })?;

        Ok(settings)
    }

    /// Makes any paths found in the Settings into absolute ones.
    ///
    /// We have a post-condition that any paths returned from the configuration
    /// db to the wider program will always be absolute paths, and this
    /// method helps to enforce that.
    ///
    /// Returns the number of settings changed.
    pub fn make_absolute(&mut self, within_dir: &Path) -> usize {
        let mut num_made_abs = 0;

        if self.default_downloads_directory.make_absolute(within_dir) {
            info!(
                "Settings: Made 'default_downloads_directory' absolute, is now {}",
                self.default_downloads_directory.to_string_lossy()
            );
            num_made_abs += 1;
        }

        num_made_abs
    }

    /// Saves the settings to the database.
    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn().execute(
            r#"UPDATE settings SET
                nick_name = ?1,
                default_downloads_directory = ?2,
                updated = ?3
            "#,
            params![self.nick_name, self.default_downloads_directory, DatabaseTime::now()],
        )?;

        info!("Saved Settings to the settings table");
        Ok(())
    }
}
