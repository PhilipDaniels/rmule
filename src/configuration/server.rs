use super::sqlite_extensions::{DatabaseTime, RowExtensions};
use super::ConfigurationDb;
use crate::parsers::ParsedServer;
use anyhow::{bail, Result};
use bitflags::bitflags;
use rusqlite::{params, Row, Statement, ToSql};
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Deref;
use tracing::info;

#[derive(Debug)]
pub struct ServerList {
    servers: Vec<Server>,
}

impl ServerList {
    /// Loads all the servers from the configuration database.
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

    /// Merges a set of parsed servers (from server.met files) into the
    /// server list. Servers are matched on ip_appr.
    pub fn merge_parsed_servers(&mut self, parsed_servers: &[ParsedServer]) {
        for ps in parsed_servers {
            if let Some(idx) = self.servers.iter().position(|s| s.ip_addr == ps.ip_addr) {
                let s = self.servers.get_mut(idx).unwrap();
                s.update_from(ps);
            } else {
                self.servers.push(ps.into());
            }
        }
    }

    pub fn save_all(&mut self, db: &ConfigurationDb) -> Result<()> {
        let conn = db.conn();
        let mut insert_stmt = conn.prepare(
            r#"INSERT INTO server
                  (
                  source, active, ip_addr, port, name,
                  description, user_count, low_id_user_count, max_user_count, ping_ms,
                  file_count, soft_file_limit, hard_file_limit, udp_flags, version,
                  last_ping_time, udp_key, udp_key_ip_addr, tcp_obfuscation_port, udp_obfuscation_port,
                  dns_name, priority, aux_ports_list, fail_count
                  )
                VALUES
                  (
                    ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20,
                    ?21, ?22, ?23, ?24
                  )
               RETURNING id"#,
        )?;

        let mut update_stmt = conn.prepare(
            r#"UPDATE server SET
                source = ?1,
                active = ?2,
                ip_addr = ?3,
                port = ?4,
                name = ?5,
                description = ?6,
                user_count = ?7,
                low_id_user_count = ?8,
                max_user_count = ?9,
                ping_ms = ?10,
                file_count = ?11,
                soft_file_limit = ?12,
                hard_file_limit = ?13,
                udp_flags = ?14,
                version = ?15,
                last_ping_time = ?16,
                udp_key = ?17,
                udp_key_ip_addr = ?18,
                tcp_obfuscation_port = ?19,
                udp_obfuscation_port = ?20,
                dns_name = ?21,
                priority = ?22,
                aux_ports_list = ?23,
                fail_count = ?24,
                updated = ?25
               WHERE
                ip_addr = ?26;"#,
        )?;

        for server in &mut self.servers {
            if server.id == 0 {
                let id = Self::insert_server(&mut insert_stmt, &server)?;
                server.id = id;
            } else {
                Self::update_server(&mut update_stmt, &server)?;
            }
        }

        Ok(())
    }

    fn update_server(stmt: &mut Statement, server: &Server) -> Result<()> {
        Ok(())
    }

    fn insert_server(stmt: &mut Statement, server: &Server) -> Result<u32> {
        //let row = ServerRow::new(server);
        let params = Self::get_params2(server);
        let mut rows = stmt.query([])?;

        match rows.next()? {
            Some(row) => {
                let id: u32 = row.get("id")?;
                Ok(id)
            }
            None => bail!("Insert of {} to server table failed", server.ip_addr),
        }
    }

    fn get_params2(server: &Server) -> Vec<Box<&dyn ToSql>> {
        let mut v: Vec<Box<&dyn ToSql>> = Vec::new();

        v.push(Box::new(&server.ip_addr.to_string()));

        v
    }

    fn get_params<'a>(row: &'a ServerRow) -> &'a [&'a dyn ToSql] {
        params![row.xip_addr]

        /*
        let udp_flags = match server.udp_flags {
            Some(p) => Some(p.bits),
            None => None,
        };

        let priority = match &server.priority {
            Some(p) => Some(*p as u32),
            None => None,
        };

        let aux_ports_list = String::new();
        for port in &server.aux_ports_list {}
        //: String = server.aux_ports_list.iter().map(|port| port.to_string());

        let ip_addr = server.ip_addr.to_string();
        let udp_key_ip_addr = match server.udp_key_ip_addr {
            Some(ip) => Some(ip.to_string()),
            None => None,
        };

        params![
            server.source,
            server.active,
            ip_addr,
            server.port,
            server.name,
            server.description,
            server.user_count,
            server.low_id_user_count,
            server.max_user_count,
            server.ping_ms,
            server.file_count,
            server.soft_file_limit,
            server.hard_file_limit,
            udp_flags,
            server.version,
            server.last_ping_time,
            server.udp_key,
            udp_key_ip_addr,
            server.tcp_obfuscation_port,
            server.udp_obfuscation_port,
            server.dns_name,
            priority,
            aux_ports_list,
            server.fail_count,
        ]
        */
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
    ip_addr: IpAddr,
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
    udp_key: Option<u32>,
    /// UNKNOWN
    udp_key_ip_addr: Option<IpAddr>,
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

/// Internal struct to help with saving to the database.
struct ServerRow<'a> {
    server: &'a Server,
    xip_addr: String,
    udp_flags: Option<u32>,
    priority: Option<u32>,
    udp_key_ip_addr: Option<String>,
}

impl<'a> ServerRow<'a> {
    fn new(server: &'a Server) -> Self {
        ServerRow {
            server,
            xip_addr: server.ip_addr.to_string(),
            udp_flags: match server.udp_flags {
                Some(p) => Some(p.bits),
                None => None,
            },
            priority: match server.priority {
                Some(p) => Some(p as u32),
                None => None,
            },
            udp_key_ip_addr: match server.udp_key_ip_addr {
                Some(ip) => Some(ip.to_string()),
                None => None,
            },
        }
    }
}

impl<'a> Deref for ServerRow<'a> {
    type Target = Server;

    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: Default::default(),
            source: Default::default(),
            active: Default::default(),
            ip_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED).into(),
            port: Default::default(),
            name: Default::default(),
            description: Default::default(),
            user_count: Default::default(),
            low_id_user_count: Default::default(),
            max_user_count: Default::default(),
            ping_ms: Default::default(),
            file_count: Default::default(),
            soft_file_limit: Default::default(),
            hard_file_limit: Default::default(),
            udp_flags: Default::default(),
            version: Default::default(),
            last_ping_time: Default::default(),
            udp_key: Default::default(),
            udp_key_ip_addr: Default::default(),
            tcp_obfuscation_port: Default::default(),
            udp_obfuscation_port: Default::default(),
            dns_name: Default::default(),
            priority: Default::default(),
            aux_ports_list: Default::default(),
            fail_count: Default::default(),
        }
    }
}

impl From<&ParsedServer> for Server {
    fn from(value: &ParsedServer) -> Self {
        let mut s = Self::default();
        s.update_from(value);
        s.id = 0;
        s.source = "???".to_owned();
        s.active = true;
        s.ip_addr = value.ip_addr;
        s
    }
}

impl Server {
    fn update_from(&mut self, ps: &ParsedServer) {
        self.port = ps.port;
        self.name = ps.name.clone();
        self.description = ps.description.clone();
        self.user_count = ps.user_count;
        self.low_id_user_count = ps.low_id_user_count;
        self.max_user_count = ps.max_user_count;
        self.ping_ms = ps.ping;
        self.file_count = ps.file_count;
        self.soft_file_limit = ps.soft_file_limit;
        self.hard_file_limit = ps.hard_file_limit;
        //self.udp_flags = ps.udp_flags;
        self.version = ps.version.clone();
        //self.last_ping_time = ps.last_ping_time;
        self.udp_key = ps.udp_key;
        self.udp_key_ip_addr = ps.udp_key_ip_addr;
        self.tcp_obfuscation_port = ps.tcp_obfuscation_port;
        self.udp_obfuscation_port = ps.udp_obfuscation_port;
        self.dns_name = ps.dns.clone();
        //self.priority = ps.preference;
        //self.aux_ports_list = ps.aux_ports_list;
        self.fail_count = ps.fail_count;
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
            ip_addr: row
                .get_ip_addr("ip_addr")?
                .expect("server.ip_addr is a mandatory field"),
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
            udp_key_ip_addr: row.get_ip_addr("udp_key_ip_addr")?,
            tcp_obfuscation_port: row.get("tcp_obfuscation_port")?,
            udp_obfuscation_port: row.get("udp_obfuscation_port")?,
            dns_name: row.get("dns_name")?,
            priority: Some(priority),
            aux_ports_list: aux_ports,
            fail_count: row.get("fail_count")?,
        })
    }
}

/// Server priority.
#[derive(Debug, Copy, Clone)]
pub enum ServerPriority {
    Low = 0,
    Normal = 1,
    High = 2,
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
