use super::ConfigurationDb;
use anyhow::Result;
use rusqlite::Row;
use tracing::info;

/// The rmule equivalent of addresses.dat from emule.
/// This is a list of addresses from which server.met files
/// can be downloaded.
#[derive(Debug)]
pub struct AddressList {
    addresses: Vec<Address>,
}

/// An address from which a server.met file can be downloaded.
#[derive(Debug)]
pub struct Address {
    pub id: u32,
    pub url: String,
    pub active: bool,
}

impl TryFrom<&Row<'_>> for Address {
    type Error = rusqlite::Error;

    /// Convert a Rusqlite row to an Address value.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("ïd")?,
            url: row.get("url")?,
            active: row.get("active")?,
        })
    }
}

impl AddressList {
    /// Load all addresses from the database.
    pub fn load_all(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT id, url, active FROM address")?;

        let addresses: Vec<_> = stmt
            .query_map([], |row| Address::try_from(row))?
            .flatten()
            .collect();

        info!("Loaded {} rows from address", addresses.len());

        Ok(Self { addresses })
    }
}
