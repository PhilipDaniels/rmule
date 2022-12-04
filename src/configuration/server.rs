use anyhow::Result;
use std::net::{self, IpAddr};
use std::str::FromStr;

use tracing::info;

use super::ConfigurationDb;

#[derive(Debug)]
pub struct ServerList {
    servers: Vec<Server>,
}

#[derive(Debug)]
pub struct Server {
    id: u32,
    address: IpAddr,
}

impl ServerList {
    pub fn load_all(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT * FROM server")?;

        let mut servers: Vec<Server> = stmt
            .query_map([], |row| {
                let ip: String = row.get("address")?;
                // TODO: Get rid of this.
                let ip = IpAddr::from_str(&ip).expect("a valid IP");
                Ok(Server { id: row.get("id")?, address: ip })
            })?
            .flatten()
            .collect();

        if servers.is_empty() {
            // We need 1 server at least.
        }

        info!("Loaded {} rows from servers", servers.len());

        Ok(Self { servers })
    }
}
