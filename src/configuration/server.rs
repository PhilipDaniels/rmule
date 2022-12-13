use super::{ConfigurationDb, IpAddr};
use crate::parsers::ParsedServer;
use crate::times;
use crate::utils::{SliceExtensions, StringExtensions};
use anyhow::{bail, Result};
use bitflags::bitflags;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput};
use rusqlite::{params, Row, Statement, ToSql};
use time::OffsetDateTime;
use tracing::info;

#[derive(Debug)]
pub struct ServerList {
    servers: Vec<Server>,
}

/// Represents a server on the ed2k network. Only the IP Address and port
/// are mandatory to establish a connection to a server, however most of
/// the other fields are usually provided in a server.met file.
/// See http://wiki.amule.org/t/index.php?title=Server.met_file
#[derive(Debug, Default)]
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
    udp_flags: Option<ServerUdpFlags>,
    /// Version and name of the software the server is running to support the
    /// ed2k network.
    version: Option<String>,
    /// The last time the server was pinged.
    last_ping_time: Option<OffsetDateTime>,
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
    /// This can be an empty list.
    aux_ports_list: Vec<u16>,
    /// How many times connecting to the server failed (reset to 0 on success?)
    fail_count: Option<u32>,
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
        let mut num_inserted = 0;
        let mut num_updated = 0;

        for ps in parsed_servers {
            if let Some(idx) = self.servers.iter().position(|s| *s.ip_addr == ps.ip_addr) {
                let s = self.servers.get_mut(idx).unwrap();
                s.update_from(ps);
                num_updated += 1;
            } else {
                self.servers.push(ps.into());
                num_inserted += 1;
            }
        }

        info!("Updated {num_updated} existing servers, inserted {num_inserted} new ones");
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
                port = ?3,
                name = ?4,
                description = ?5,
                user_count = ?6,
                low_id_user_count = ?7,
                max_user_count = ?8,
                ping_ms = ?9,
                file_count = ?10,
                soft_file_limit = ?11,
                hard_file_limit = ?12,
                udp_flags = ?13,
                version = ?14,
                last_ping_time = ?15,
                udp_key = ?16,
                udp_key_ip_addr = ?17,
                tcp_obfuscation_port = ?18,
                udp_obfuscation_port = ?19,
                dns_name = ?20,
                priority = ?21,
                aux_ports_list = ?22,
                fail_count = ?23,
                updated = ?24
               WHERE
                ip_addr = ?25;"#,
        )?;

        for server in &mut self.servers {
            if server.id == 0 {
                let id = Self::insert_server(&mut insert_stmt, server)?;
                server.id = id;
            } else {
                Self::update_server(&mut update_stmt, server)?;
            }
        }

        Ok(())
    }

    fn update_server(stmt: &mut Statement, server: &Server) -> Result<()> {
        let params = params![
            server.source,
            server.active,
            //server.ip_addr,
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
            server.udp_flags,
            server.version,
            server.last_ping_time,
            server.udp_key,
            server.udp_key_ip_addr,
            server.tcp_obfuscation_port,
            server.udp_obfuscation_port,
            server.dns_name,
            server.priority,
            server.aux_ports_list.to_comma_string(),
            server.fail_count,
            times::now_to_sql(),
            server.ip_addr,
        ];

        let row_count = stmt.execute(params)?;

        if row_count != 1 {
            bail!(
                "Update of server with ip {} in server table failed",
                server.ip_addr
            );
        }

        Ok(())
    }

    fn insert_server(stmt: &mut Statement, server: &Server) -> Result<u32> {
        let params = params![
            server.source,
            server.active,
            server.ip_addr,
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
            server.udp_flags,
            server.version,
            server.last_ping_time,
            server.udp_key,
            server.udp_key_ip_addr,
            server.tcp_obfuscation_port,
            server.udp_obfuscation_port,
            server.dns_name,
            server.priority,
            server.aux_ports_list.to_comma_string(),
            server.fail_count
        ];

        let mut rows = stmt.query(params)?;

        match rows.next()? {
            Some(row) => {
                let id: u32 = row.get("id")?;
                Ok(id)
            }
            None => bail!(
                "Insert of server with ip {} to server table failed",
                server.ip_addr
            ),
        }
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

impl From<&ParsedServer> for Server {
    fn from(value: &ParsedServer) -> Self {
        let mut s = Self::default();
        s.update_from(value);
        s.id = 0;
        s.source = "???".to_owned();
        s.active = true;
        s.ip_addr = value.ip_addr.into();
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
        self.udp_flags = ps.udp_flags;
        self.version = ps.version.clone();
        //self.last_ping_time = ps.last_ping_time;
        self.udp_key = ps.udp_key;
        self.udp_key_ip_addr = ps.udp_key_ip_addr.map(|addr| addr.into());
        self.tcp_obfuscation_port = ps.tcp_obfuscation_port;
        self.udp_obfuscation_port = ps.udp_obfuscation_port;
        self.dns_name = ps.dns.clone();
        self.priority = ps.priority;
        self.aux_ports_list = ps.aux_ports_list.clone().unwrap_or_default();
        self.fail_count = ps.fail_count;
    }
}

impl TryFrom<&Row<'_>> for Server {
    type Error = anyhow::Error;

    /// Build a Server value from a Rusqlite Row.
    /// This must return a rusqlite::Error in order to be usable
    /// from within QueryMap.
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let ports = match row.get::<_, Option<String>>("aux_ports_list")? {
            Some(s) => s.split_comma_str_to_vec()?,
            None => Vec::new(),
        };

        Ok(Self {
            id: row.get("id")?,
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
            udp_flags: row.get("udp_flags")?,
            version: row.get("version")?,
            last_ping_time: row.get("last_ping_time")?,
            udp_key: row.get("udp_key")?,
            udp_key_ip_addr: row.get("udp_key_ip_addr")?,
            tcp_obfuscation_port: row.get("tcp_obfuscation_port")?,
            udp_obfuscation_port: row.get("udp_obfuscation_port")?,
            dns_name: row.get("dns_name")?,
            priority: row.get("priority")?,
            aux_ports_list: ports,
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
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        ServerPriority::try_from(value as i64)
    }
}

impl TryFrom<i64> for ServerPriority {
    type Error = anyhow::Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Normal),
            1 => Ok(Self::High),
            2 => Ok(Self::Low),
            _ => bail!(
                "The value {value} is outside the expected range (0, 1 or 2 for ServerPriority"
            ),
        }
    }
}

impl ToSql for ServerPriority {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let n = *self as u32;
        Ok(ToSqlOutput::from(n))
    }
}

impl FromSql for ServerPriority {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_i64().and_then(|n| {
            let sp = match ServerPriority::try_from(n) {
                Ok(sp) => sp,
                Err(_) => return Err(FromSqlError::OutOfRange(n)),
            };
            FromSqlResult::Ok(sp)
        })
    }
}

bitflags! {
    /// What actions are supported via UDP.
    pub struct ServerUdpFlags: u32 {
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

impl From<u32> for ServerUdpFlags {
    fn from(value: u32) -> Self {
        // Always convert, throw away any bits we don't understand.
        ServerUdpFlags::from_bits_truncate(value)
    }
}

impl ToSql for ServerUdpFlags {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.bits))
    }
}

impl FromSql for ServerUdpFlags {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_i64().and_then(|n| {
            // Slightly nasty cast, but in practice safe.
            let sp = match ServerUdpFlags::try_from(n as u32) {
                Ok(sp) => sp,
                Err(_) => return Err(FromSqlError::OutOfRange(n)),
            };
            FromSqlResult::Ok(sp)
        })
    }
}
