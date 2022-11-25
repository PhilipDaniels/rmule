use super::ConfigurationDb;
use super::sqlite_extensions::DatabaseTime;
use anyhow::Result;
use rusqlite::params;

#[derive(Debug)]
pub struct Settings {
    pub nick_name: String,
}

impl Settings {
    /// Loads the settings from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let mut settings = db.conn().query_row("SELECT * FROM settings", [], |row| {
            Ok(Self {
                nick_name: row.get("nick_name")?,
            })
        })?;

        Ok(settings)
    }

    /// Saves the settings to the database.
    pub fn save(&self, db: &ConfigurationDb) -> Result<()> {
        db.conn().execute(
            r#"UPDATE settings SET
                nick_name = ?1,
                updated = ?2
            "#,
            params![
                self.nick_name,
                DatabaseTime::now()
            ],
        )?;

        Ok(())
    }

    // Ensures that all the paths on any settings object are all absolute paths.
    // fn make_paths_absolute(&mut self, dir: &Path) -> bool {
    //     let mut did_change = false;

    //     match file::make_absolute(&self.downloaded_directory, dir) {
    //         Cow::Borrowed(_) => {}
    //         Cow::Owned(p) => {
    //             self.downloaded_directory = p;
    //             did_change = true;
    //         }
    //     }

    //     did_change
    // }
}
