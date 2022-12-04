use anyhow::Result;
use std::net::IpAddr;

use tracing::info;

use super::sqlite_extensions::DatabaseTime;
use super::ConfigurationDb;

#[derive(Debug)]
pub struct ServerList {
    servers: Vec<Server>,
}

#[derive(Debug)]
pub struct Server {
    /// The Id of the server, from the database table.
    id: u32,
    /// The IP Address of the server.
    ip: IpAddr,
    /// The port through which rMule will connect to the server.
    /// See also aux_ports_list.
    port: u16,
    /// The name of the server.
    name: String,
    /// Short description of the server.
    description: String,
    /// Time (in ms) it takes to communicate with the server.
    ping: u32,
    /// How many times connecting to the server failed (reset to 0 on success?)
    fail_count: u32,
    /// Server priority.
    priority: ServerPriority,
    /// The DNS name of the server.
    dns: String,
    /// Maximum number of users the server allows to simultaneously connect
    max_users: u32,
    /// Soft files is the minimum number of files you must share to not be
    /// penalized.
    soft_files: u32,
    /// Hard files is the maximum number of files you must share to not be
    /// penalized.
    hard_files: u32,
    /// The last time the server was pinged.
    last_ping_time: DatabaseTime,
    /// Version and name of the software the server is running to support the
    /// ed2k network.
    version: String,
    /// What actions are supported via UDP.
    udp_flags: ServerUdpActions,
    /// List of auxiliary ports which can be tried if the standard one fails.
    aux_ports_list: Vec<String>,
    /// The number of users currently registered on the server.
    users: u32,
    /// The number of files registered on the server.
    files: u32,
    /// The download URL or "manual" from where this server originated.
    source: String,
    /// A flag to indicate whether the server is active. This allows us to
    /// disable servers without removing them from the list and losing them.
    active: bool,
}

/// Server priority.
#[derive(Debug)]
pub enum ServerPriority {
    Low,
    Normal,
    High,
}

/// What actions are supported via UDP.
#[derive(Debug)]
pub enum ServerUdpActions {
    GetSources = 0x01,
    GetFiles = 0x02,
    NewTags = 0x08,
    Unicode = 0x10,
    GetExtendedSourcesInfo = 0x20,
}

impl ServerList {
    pub fn load_all(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT * FROM server")?;

        let mut servers: Vec<Server> = Vec::new();

        // let mut servers: Vec<Server> = stmt
        //     .query_map([], |row| {
        //         let ip: String = row.get("address")?;
        //         // TODO: Get rid of this.
        //         let ip = IpAddr::from_str(&ip).expect("a valid IP");
        //         Ok(Server { id: row.get("id")?, address: ip })
        //     })?
        //     .flatten()
        //     .collect();

        if servers.is_empty() {
            // We need 1 server at least.
        }

        info!("Loaded {} rows from servers", servers.len());

        Ok(Self { servers })
    }
}
