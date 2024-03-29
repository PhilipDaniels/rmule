use crate::times;
use anyhow::Result;
use rusqlite::{params, Connection, Row, Statement};
use time::OffsetDateTime;
use tracing::{info, warn};

/// The rmule equivalent of addresses.dat from emule.
/// This is a list of addresses from which server.met files
/// can be downloaded.
#[derive(Debug, Clone)]
pub struct AddressList {
    addresses: Vec<Address>,
}

/// An address from which a server.met file can be downloaded.
#[derive(Debug, Clone)]
pub struct Address {
    pub created: OffsetDateTime,
    pub updated: OffsetDateTime,
    pub id: i64,
    // URL from which to fetch server.met files.
    pub url: String,
    // Short description of the address.
    pub description: String,
    // Logical delete flag.
    pub active: bool,
}

impl TryFrom<&Row<'_>> for Address {
    type Error = rusqlite::Error;

    /// Build an Address value from a Rusqlite Row.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            created: row.get("created")?,
            updated: row.get("updated")?,
            id: row.get("id")?,
            url: row.get("url")?,
            description: row.get("description")?,
            active: row.get("active")?,
        })
    }
}

impl Address {
    pub fn new<S: Into<String>>(url: S, description: S, active: bool) -> Self {
        Self {
            created: times::now(),
            updated: times::now(),
            id: 0,
            url: url.into(),
            description: description.into(),
            active,
        }
    }
}

impl AddressList {
    /// Inserts a reasonable set of default addresses.
    pub fn insert_default_addresses(&mut self, conn: &Connection) -> Result<()> {
        #[rustfmt::skip]
        const DEFAULT_ADDRESSES: [(&str, &str); 1] = [
            ("http://upd.emule-security.org/server.met", "DEFAULT RMULE ADDRESS"),
            // ("http://www.gruk.org/server.met.gz", "DEFAULT RMULE ADDRESS"),
            // ("http://peerates.net/server.met", "DEFAULT RMULE ADDRESS)"),
            // ("http://shortypower.dyndns.org/server.met", "DEFAULT RMULE ADDRESS"),
            // ("http://www.server-met.de/dl.php?load=gz", "DEFAULT RMULE ADDRESS, Curated (best) from this site"),
            // ("http://www.server-met.de/dl.php?load=min", "DEFAULT RMULE ADDRESS, Curated (medium) from this site"),
            // ("http://www.server-met.de/dl.php?load=max", "DEFAULT ARMULE DDRESS, Curated (All) from this site"),
            // ("http://ed2k.2x4u.de/v1s4vbaf/micro/server.met", "DEFAULT RMULE ADDRESS, Curated (Connect List) from this site"),
            // ("http://ed2k.2x4u.de/v1s4vbaf/min/server.met", "DEFAULT RMULE ADDRESS, Curated (Best) from this site"),
            // ("http://ed2k.2x4u.de/v1s4vbaf/max/server.met", "DEFAULT RMULE ADDRESS, Curated (All) from this site"),
        ];

        info!("Inserting reasonable default addresses (as of Dec 2022)");

        for addr in DEFAULT_ADDRESSES.iter() {
            let a = Address::new(addr.0, addr.1, true);
            self.insert(conn, a)?;
        }

        Ok(())
    }

    /// Load all addresses from the database.
    pub fn load_all(conn: &Connection) -> Result<Self> {
        let mut stmt = conn.prepare("SELECT * FROM address")?;

        let addresses: Vec<_> = stmt
            .query_map([], |row| Address::try_from(row))?
            .flatten()
            .collect();

        let addresses = Self { addresses };
        info!("Loaded {} rows from address", addresses.len());
        Ok(addresses)
    }

    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.addresses.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Address> {
        self.into_iter()
    }

    pub fn insert(&mut self, conn: &Connection, mut address: Address) -> Result<()> {
        if let Some(existing) = self
            .addresses
            .iter()
            .find(|a| a.url.to_lowercase() == address.url.to_lowercase())
        {
            warn!(
                "Address {} is already in the address list with id of {}, ignoring",
                existing.url, existing.id
            );
            return Ok(());
        }

        address.created = times::now();
        address.updated = address.created;

        let params = params![
            address.created,
            address.updated,
            address.url,
            address.description,
            address.active
        ];

        let mut stmt = Self::insert_stmt(conn)?;
        stmt.execute(params)?;
        address.id = conn.last_insert_rowid();
        info!(
            "Inserted address {} with url of {}",
            address.id, address.url
        );
        self.addresses.push(address);
        Ok(())
    }

    pub fn update(&mut self, conn: &Connection, address: &mut Address) -> Result<()> {
        address.updated = times::now();

        let params = params![
            address.updated,
            address.url,
            address.description,
            address.active,
            address.id
        ];

        let mut stmt = Self::update_stmt(conn)?;
        stmt.execute(params)?;

        info!("Updated address {} with url of {}", address.id, address.url);

        Ok(())
    }

    fn update_stmt(conn: &Connection) -> Result<Statement<'_>> {
        Ok(conn.prepare(
            r#"UPDATE address SET
                updated = ?1,
                url = ?2,
                description = ?3,
                active = ?4
            WHERE
                id = ?5"#,
        )?)
    }

    fn insert_stmt(conn: &Connection) -> Result<Statement<'_>> {
        Ok(conn.prepare(
            r#"INSERT INTO address(created, updated, url, description, active)
               VALUES (?1, ?2, ?3, ?4, ?5)"#,
        )?)
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
