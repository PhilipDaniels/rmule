use super::sqlite_extensions::{DatabasePathBuf, DatabaseTime};
use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::{params, Row};
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub nick_name: String,
    /// Default downloads directory to be used if not set on the TempDirectory
    /// or on the download itself.
    pub default_downloads_directory: DatabasePathBuf,
    pub auto_update_server_list: bool,
}

impl TryFrom<&Row<'_>> for Settings {
    type Error = rusqlite::Error;

    /// Build a Settings value from a Rusqlite Row.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            nick_name: row.get("nick_name")?,
            default_downloads_directory: row.get("default_downloads_directory")?,
            auto_update_server_list: row.get("auto_update_server_list")?,
        })
    }
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT * FROM settings")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Settings::try_from(row)?)
        } else {
            info!("No settings rows in database, creating default");
            let ddir_pb = dirs::download_dir().unwrap_or_else(|| "Downloads".into());
            let default_settings = Self {
                nick_name: "http://www.rMule.org".to_owned(),
                default_downloads_directory: ddir_pb.into(),
                auto_update_server_list: true,
            };

            default_settings.insert(db)?;
            Ok(default_settings)
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
                auto_update_server_list = ?3
                updated = ?4
            "#,
            params![
                self.nick_name,
                self.default_downloads_directory,
                self.auto_update_server_list,
                DatabaseTime::now()
            ],
        )?;

        info!("Saved Settings to the settings table");
        Ok(())
    }

    pub fn insert(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn().execute(
            r#"INSERT INTO settings(nick_name, default_downloads_directory, auto_update_server_list)
                VALUES(?1, ?2, ?3);
            "#,
            params![
                self.nick_name,
                self.default_downloads_directory,
                self.auto_update_server_list
            ],
        )?;

        info!("Inserted Settings to the settings table");
        Ok(())
    }
}
