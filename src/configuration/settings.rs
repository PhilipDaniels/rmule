use std::path::Path;

use super::sqlite_extensions::{DatabasePathBuf, DatabaseTime};
use super::ConfigurationDb;
use anyhow::{bail, Result};
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
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT * FROM settings")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Self {
                nick_name: row.get("nick_name")?,
                default_downloads_directory: row.get("default_downloads_directory")?,
            })
        } else {
            info!("No settings rows in database, creating default");
            let ddir_pb = dirs::download_dir().unwrap_or("Downloads".into());
            let default_settings = Self {
                nick_name: "http://www.rMule.org".to_owned(),
                default_downloads_directory: ddir_pb.into(),
            };

            default_settings.insert(db)?;
            return Ok(default_settings);
        }
    }

    /*
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
    */

    /// Updates existing settings in the database.
    pub fn update(&self, db: &ConfigurationDb) -> Result<()> {
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

    pub fn insert(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn().execute(
            r#"INSERT INTO settings(nick_name, default_downloads_directory)
                VALUES(?1, ?2);
            "#,
            params![self.nick_name, self.default_downloads_directory],
        )?;

        info!("Inserted Settings to the settings table");
        Ok(())
    }
}
