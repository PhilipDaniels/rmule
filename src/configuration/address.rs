use super::ConfigurationDb;
use anyhow::{bail, Result};
use rusqlite::{params, Row};
use tracing::{info, warn};

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
    pub description: String,
    pub active: bool,
}

impl TryFrom<&Row<'_>> for Address {
    type Error = rusqlite::Error;

    /// Convert a Rusqlite row to an Address value.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.get("ïd")?,
            url: row.get("url")?,
            description: row.get("description")?,
            active: row.get("active")?,
        })
    }
}

impl AddressList {
    /// Load all addresses from the database.
    pub fn load_all(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT id, url, description, active FROM address")?;

        let addresses: Vec<_> = stmt
            .query_map([], |row| Address::try_from(row))?
            .flatten()
            .collect();

        if addresses.is_empty() {
            info!("No addresses found, populating reasonable defaults (as of Dec 2022)");
            let mut addresses = Self {
                addresses: Vec::new(),
            };

            addresses.insert_address(db, "http://www.gruk.org/server.met.gz", "DEFAULT")?;
            addresses.insert_address(db, "http://peerates.net/server.met", "DEFAULT")?;
            addresses.insert_address(db, "http://shortypower.dyndns.org/server.met", "DEFAULT")?;

            // // 3 files from http://www.server-met.de/
            addresses.insert_address(
                db,
                "http://www.server-met.de/dl.php?load=gz",
                "DEFAULT, Curated (best) from this site",
            )?;
            addresses.insert_address(
                db,
                "http://www.server-met.de/dl.php?load=min",
                "DEFAULT, Curated (medium) from this site",
            )?;
            addresses.insert_address(
                db,
                "http://www.server-met.de/dl.php?load=max",
                "DEFAULT, Curated (All) from this site",
            )?;

            // // 3 files from http://ed2k.2x4u.de/index.html
            addresses.insert_address(
                db,
                "http://ed2k.2x4u.de/v1s4vbaf/micro/server.met",
                "DEFAULT, Curated (All) from this site",
            )?;

            addresses.insert_address(
                db,
                "http://ed2k.2x4u.de/v1s4vbaf/min/server.met",
                "DEFAULT, Curated (All) from this site",
            )?;

            addresses.insert_address(
                db,
                "http://ed2k.2x4u.de/v1s4vbaf/max/server.met",
                "DEFAULT, Curated (All) from this site",
            )?;

            Ok(addresses)
        } else {
            info!("Loaded {} rows from address", addresses.len());
            Ok(Self { addresses })
        }
    }

    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.addresses.len() == 0
    }

    pub fn insert_address(
        &mut self,
        db: &ConfigurationDb,
        url: &str,
        description: &str,
    ) -> Result<()> {
        if self
            .addresses
            .iter()
            .any(|a| a.url.to_lowercase() == url.to_lowercase())
        {
            warn!("Address {} is already in the address list, ignoring", url);
            return Ok(());
        }

        let conn = db.conn();
        let mut stmt = conn.prepare(
            r#"INSERT INTO address(url, description, active)
               VALUES (?1, ?2, ?3)
               RETURNING id"#,
        )?;

        let mut rows = stmt.query(params![url, description, 1])?;
        let id = rows
            .next()?
            .expect("Insert to address table failed")
            .get("id")?;

        let addr = Address {
            id,
            url: url.to_owned(),
            description: description.to_owned(),
            active: true,
        };

        info!("Inserted address {} with id of {}", addr.url, addr.id);

        self.addresses.push(addr);
        Ok(())
    }
}

impl IntoIterator for AddressList {
    type Item = Address;
    type IntoIter = std::vec::IntoIter<Address>;

    fn into_iter(self) -> Self::IntoIter {
        self.addresses.into_iter()
    }
}

impl<'a> IntoIterator for &'a AddressList {
    type Item = &'a Address;
    type IntoIter = std::slice::Iter<'a, Address>;

    fn into_iter(self) -> Self::IntoIter {
        self.addresses.iter()
    }
}

impl<'a> IntoIterator for &'a mut AddressList {
    type Item = &'a mut Address;
    type IntoIter = std::slice::IterMut<'a, Address>;

    fn into_iter(self) -> Self::IntoIter {
        self.addresses.iter_mut()
    }
}
