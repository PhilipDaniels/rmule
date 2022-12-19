use super::PathBuf;
use crate::times;
use anyhow::Result;
use rusqlite::{params, Connection, Row};
use time::OffsetDateTime;
use tracing::info;

#[derive(Debug)]
pub struct Settings {
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,

    /// Name we are known by on the ed2k network.
    pub nick_name: String,
    /// Default downloads directory to be used if not set on the TempDirectory
    /// or on the download itself.
    pub default_downloads_directory: PathBuf,
    /// Whether to automatically update the list of servers when rMule starts.
    pub auto_update_server_list: bool,
}

impl TryFrom<&Row<'_>> for Settings {
    type Error = rusqlite::Error;

    /// Build a Settings value from a Rusqlite Row.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            created: row.get("created")?,
            updated: row.get("updated")?,
            nick_name: row.get("nick_name")?,
            default_downloads_directory: row.get("default_downloads_directory")?,
            auto_update_server_list: row.get("auto_update_server_list")?,
        })
    }
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(conn: &Connection) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM settings")?;
        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Settings::try_from(row)?)
        } else {
            info!("No settings rows in database, creating default");
            let now = times::now();
            let ddir_pb = dirs::download_dir().unwrap_or_else(|| "Downloads".into());
            let default_settings = Self {
                created: now,
                updated: now,
                nick_name: "http://www.rMule.org".to_owned(),
                default_downloads_directory: ddir_pb.into(),
                auto_update_server_list: true,
            };

            default_settings.insert(conn)?;
            Ok(default_settings)
        }
    }

    /// Updates existing settings in the database.
    pub fn update(&self, conn: &Connection) -> Result<()> {
        conn.execute(
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
                times::now(),
            ],
        )?;

        info!("Saved Settings to the settings table");
        Ok(())
    }

    pub fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            r#"INSERT INTO settings(created, updated, nick_name, default_downloads_directory, auto_update_server_list)
                VALUES(?1, ?2, ?3, ?4, ?5);
            "#,
            params![
                self.created,
                self.updated,
                self.nick_name,
                self.default_downloads_directory,
                self.auto_update_server_list,
            ],
        )?;

        info!("Inserted Settings to the settings table");
        Ok(())
    }
}
