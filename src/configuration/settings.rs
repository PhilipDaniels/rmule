use super::sqlite_extensions::{DatabasePathBuf, DatabaseTime};
use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::params;

pub struct Settings {
    pub nick_name: String,
    /// When a download is started it is written to
    /// 1. The directory set for the download itself, if set.
    /// 2. This directory, if set.
    /// 3. The default default_completed_directory on the Settings,
    /// which is always set.
    pub default_completed_directory: DatabasePathBuf,
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let mut settings = db.conn().query_row("SELECT * FROM settings", [], |row| {
            Ok(Self {
                nick_name: row.get("nick_name")?,
                default_completed_directory: row.get("default_completed_directory")?,
            })
        })?;

        if settings
            .default_completed_directory
            .make_absolute(&db.config_dir)
        {
            eprintln!("Settings: Saving to db after making 'default_completed_directory' absolute, is now {}",
                settings.default_completed_directory.to_string_lossy());

            settings.save(db)?;
        }

        Ok(settings)
    }

    /// Saves the settings to the database.
    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn().execute(
            r#"UPDATE settings SET
                nick_name = ?1,
                default_completed_directory = ?2,
                updated = ?3
            "#,
            params![
                self.nick_name,
                self.default_completed_directory,
                DatabaseTime::now()
            ],
        )?;

        Ok(())
    }
}
