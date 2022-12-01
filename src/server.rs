use std::path::Path;

use anyhow::Result;

use serde::{Deserialize, Serialize};

use crate::file;

#[derive(Serialize, Deserialize)]
pub enum ServerPriority {
    Low,
    Normal,
    High,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub priority: ServerPriority,
    port: u16,
    ip: std::net::IpAddr,
    users: u32,
    files: u32,
    max_users: u32,
    soft_files: u32,
    hard_files: u32,
    pub is_static: bool,
    version: String,
    description: String,
    name: String,
}

pub struct ServerList {
    servers: Vec<Server>,
}

impl ServerList {
    /// Loads all the servers and returns them as a list.
    /// The servers from the server.toml file are loaded first,
    /// then the static servers from staticservers.toml are loaded.
    pub fn load(filename: &Path) -> Result<Self> {
        let mut servers = Self { servers: Vec::new() };

        if file::file_exists(filename)? {
            // Deserialize as bincode.
        }

        Ok(servers)
    }

    fn save(&self, filename: &Path) -> Result<()> {
        Ok(())
    }
}
