use std::path::{Path, PathBuf};
use anyhow::{Result};
use rusqlite::Connection;

mod migrations;
mod sqlite_extensions;

/// The version number of the 'current' database, i.e. the one
/// that this program is designed to run against. If any database
/// change is made then this number must be incremented.
//const CURRENT_DATABASE_VERSION: u16 = 1;

pub struct Configuration {
    pub config_dir: PathBuf,
    pub database_version: u16
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            config_dir: Default::default(),
            database_version: 0,
        }
    }
}

impl Configuration {
    /// Loads new configuration from disk. If the configuration database does
    /// not exist then a new one is created. If the database requires an upgrade,
    /// then that is applied first before the configuration is returned.
    /// 
    /// This function should be called at startup, once it has run then
    /// the database is accessed by individual methods as needed.
    pub fn load(config_dir: &Path) -> Result<Self> {
        let filename = Self::db_filename(config_dir);

        // This will create an empty SQLLite db if needed.
        let db = Connection::open(&filename)?;

        migrations::apply_database_migrations(&db)?;

        let mut cfg = Configuration::default();
        cfg.config_dir = config_dir.into();
        Ok(cfg)
    }

    fn db_filename<P: Into<PathBuf>>(config_dir: P) -> PathBuf {
        let mut p = config_dir.into();
        p.push("rmule.sqlite");
        p
    }



}
