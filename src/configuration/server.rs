use super::sqlite_extensions::{DatabaseIpAddr, DatabaseTime};
use super::ConfigurationDb;
use crate::parsers::ParsedServer;
use anyhow::{bail, Result};
use bitflags::bitflags;
use rusqlite::Row;
use tracing::info;

#[derive(Debug)]
pub struct ServerList {
    servers: Vec<Server>,
}

impl ServerList {
    pub fn load_all(db: &ConfigurationDb) -> Result<Self> {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT * FROM server")?;

        let mut servers = Vec::new();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            servers.push(Server::try_from(row)?);
        }

        info!("Loaded {} rows from servers", servers.len());

        Ok(Self { servers })
    }

    pub fn merge_parsed_servers(&mut self, servers: &[ParsedServer]) {}

    pub fn save_all(&mut self, db: &ConfigurationDb) -> Result<()> {
        Ok(())
    }
}

impl IntoIterator for ServerList {
    type Item = Server;
    type IntoIter = std::vec::IntoIter<Server>;

    fn into_iter(self) -> Self::IntoIter {
        self.servers.into_iter()
    }
}

impl<'a> IntoIterator for &'a ServerList {
    type Item = &'a Server;
    type IntoIter = std::slice::Iter<'a, Server>;

    fn into_iter(self) -> Self::IntoIter {
        self.servers.iter()
    }
}

impl<'a> IntoIterator for &'a mut ServerList {
    type Item = &'a mut Server;
    type IntoIter = std::slice::IterMut<'a, Server>;

    fn into_iter(self) -> Self::IntoIter {
        self.servers.iter_mut()
    }
}

/// Represents a server on the ed2k network. Only the IP Address and port
/// are mandatory to establish a connection to a server, however most of
/// the other fields are usually provided in a server.met file.
/// See http://wiki.amule.org/t/index.php?title=Server.met_file
#[derive(Debug)]
pub struct Server {
    /// The Id of the server, from the database table.
    id: u32,
    /// The download URL or "manual" from where this server originated.
    source: String,
    /// A flag to indicate whether the server is active. This allows us to
    /// disable servers without removing them from the list and losing them.
    active: bool,
    /// The IP Address of the server.
    ip_addr: DatabaseIpAddr,
    /// The port through which rMule will connect to the server.
    /// See also aux_ports_list.
    port: u16,
    /// The friendly name of the server, e.g. "eMule Sunrise".
    name: Option<String>,
    /// Short description of the server.
    description: Option<String>,
    /// The number of users currently registered on the server.
    user_count: Option<u32>,
    /// The number of 'Low Id' users currently registered on the server.
    /// See http://wiki.amule.org/wiki/FAQ_eD2k-Kademlia#What_is_LowID_and_HighID.3F
    low_id_user_count: Option<u32>,
    /// Maximum number of users the server allows to simultaneously connect
    max_user_count: Option<u32>,
    /// Time (in ms) it takes to communicate with the server.
    ping_ms: Option<u32>,
    /// The number of files registered on the server.
    file_count: Option<u32>,
    /// Soft files is the minimum number of files you must share to not be
    /// penalized.
    soft_file_limit: Option<u32>,
    /// Hard files is the maximum number of files you must share to not be
    /// penalized.
    hard_file_limit: Option<u32>,
    /// What actions are supported via UDP.
    udp_flags: Option<ServerUdpActions>,
    /// Version and name of the software the server is running to support the
    /// ed2k network.
    version: Option<String>,
    /// The last time the server was pinged.
    last_ping_time: Option<DatabaseTime>,
    /// UNKNOWN
    udp_key: Option<DatabaseIpAddr>,
    /// UNKNOWN
    tcp_obfuscation_port: Option<u16>,
    /// UNKNOWN
    udp_obfuscation_port: Option<u16>,
    /// The DNS name of the server.
    dns_name: Option<String>,
    /// Server priority.
    priority: Option<ServerPriority>,
    /// List of auxiliary ports which can be tried if the standard one fails.
    aux_ports_list: Vec<u16>,
    /// How many times connecting to the server failed (reset to 0 on success?)
    fail_count: Option<u32>,
}

/// Server priority.
#[derive(Debug)]
pub enum ServerPriority {
    Low,
    Normal,
    High,
}

impl TryFrom<u32> for ServerPriority {
    type Error = rusqlite::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Low),
            1 => Ok(Self::Normal),
            2 => Ok(Self::High),
            _ => Err(rusqlite::Error::IntegralValueOutOfRange(0, value as i64)),
        }
    }
}

bitflags! {
    /// What actions are supported via UDP.
    pub struct ServerUdpActions: u32 {
        const GET_SOURCES          = 0b00000000001;
        const GET_FILES            = 0b00000000010;
        const NEW_TAGS             = 0b00000001000;
        const UNICODE              = 0b00000010000;
        const GET_EXTENDED_SOURCES = 0b00000100000;
        const LARGE_FILES          = 0b00100000000;
        const UDP_OBFUSCATION      = 0b01000000000;
        const TCP_OBFUSCATION      = 0b10000000000;
    }
}

impl TryFrom<u32> for ServerUdpActions {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let flags = ServerUdpActions::from_bits(value);
        if let Some(flags) = flags {
            Ok(flags)
        } else {
            bail!(
                "The value {} is not a valid ServerUdpFlags value, it contains extra bits",
                value
            )
        }
    }
}

impl TryFrom<&Row<'_>> for Server {
    type Error = anyhow::Error;

    /// Build a Server value from a Rusqlite Row.
    /// This must return a rusqlite::Error in order to be usable
    /// from within QueryMap.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let priority = row.get::<_, u32>("priority")?;
        let priority = ServerPriority::try_from(priority)?;

        let udp_flags = row.get::<_, u32>("udp_flags")?;
        let udp_flags = ServerUdpActions::try_from(udp_flags)?;

        let aux_ports: Vec<u16>;
        if let Some(apl) = row.get::<_, Option<String>>("aux_ports_list")? {
            aux_ports = apl.split(',').map(|s| s.parse()).flatten().collect();
        } else {
            aux_ports = Vec::new();
        }

        Ok(Self {
            id: row.get("ïd")?,
            source: row.get("source")?,
            active: row.get("active")?,
            ip_addr: row.get("ip_addr")?,
            port: row.get("port")?,
            name: row.get("name")?,
            description: row.get("description")?,
            user_count: row.get("user_count")?,
            low_id_user_count: row.get("low_id_user_count")?,
            max_user_count: row.get("max_user_count")?,
            ping_ms: row.get("ping_ms")?,
            file_count: row.get("file_count")?,
            soft_file_limit: row.get("soft_file_limit")?,
            hard_file_limit: row.get("hard_file_limit")?,
            udp_flags: Some(udp_flags),
            version: row.get("version")?,
            last_ping_time: row.get("last_ping_time")?,
            udp_key: row.get("udp_key")?,
            tcp_obfuscation_port: row.get("tcp_obfuscation_port")?,
            udp_obfuscation_port: row.get("udp_obfuscation_port")?,
            dns_name: row.get("dns_name")?,
            priority: Some(priority),
            aux_ports_list: aux_ports,
            fail_count: row.get("fail_count")?,
        })
    }
}
