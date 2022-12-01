use super::ConfigurationDb;
use anyhow::Result;
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
    pub url: String,
    pub active: bool,
}

impl AddressList {
    /// Load all addresses from the database.
    pub fn load(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT active, url FROM address")?;

        let addresses: Vec<_> = stmt
            .query_map([], |row| Ok(Address { active: row.get("active")?, url: row.get("url")? }))?
            .flatten()
            .collect();

        info!("Loaded {} rows from address", addresses.len());

        Ok(Self { addresses })
    }
}
