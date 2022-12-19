use super::{Address, AddressList, ServerList, Settings, TempDirectoryList};
use crate::configuration::migrations;
use crate::configuration::parsing::{self, ParsedServer};
use crate::file;
use anyhow::{Context, Result};
use futures::future::join_all;
use rusqlite::{Connection, Transaction, TransactionBehavior};
use std::cell::{Ref, RefCell};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, warn};

pub type ConfigurationCommandSender = mpsc::Sender<ConfigurationCommand>;
pub type ConfigurationCommandReceiver = mpsc::Receiver<ConfigurationCommand>;

pub type ConfigurationEventSender = broadcast::Sender<ConfigurationEvents>;
pub type ConfigurationEventReceiver = broadcast::Receiver<ConfigurationEvents>;

/// The handle type allows commands to be sent to and events to be received
/// from the Configuration Manager.
pub struct ConfigurationManagerHandle {
    /// The cmd_sender is used to send commands to the ConfigurationManager.
    cmd_sender: ConfigurationCommandSender,
    /// The evt_sender is required so that callers can subscribe to events.
    /// It is not used to actually send any events. This is a bit strange.
    evt_sender: ConfigurationEventSender,
    // TODO: We need at least one receiver to be alive to
    // allow us to send events.
    evt_receiver: ConfigurationEventReceiver,
}

impl ConfigurationManagerHandle {
    /// Starts the Configuration Manager as a Tokio task, however it does not do
    /// anything until sent a Start command.
    pub fn new(config_dir: &Path) -> Self {
        let (cmd_sender, cmd_receiver) = mpsc::channel::<ConfigurationCommand>(32);
        let (evt_sender, evt_receiver) = broadcast::channel::<ConfigurationEvents>(32);

        // Construct a new ConfigurationManager to manage the connection
        // to the configuration database.
        let mut mgr =
            ConfigurationManager::new(evt_sender.clone(), cmd_receiver, config_dir).expect("s");

        // Move the mgr onto its own blocking thread. See tokio docs
        // on spawn_blocking. SQLite is non-async, so any work it does
        // is blocking work.
        tokio::task::Builder::new()
            .name("ConfigurationMgr")
            .spawn_blocking(move || mgr.run())
            .expect("spawn_blocking of ConfigurationMgr failed");

        ConfigurationManagerHandle {
            cmd_sender,
            evt_sender,
            evt_receiver,
        }
    }

    /// Sends a command to the ConfigurationManager.
    pub async fn send_command(&self, cmd: ConfigurationCommand) -> Result<()> {
        Ok(self.cmd_sender.send(cmd).await?)
    }

    /// Create a new subscription to events sent by the Configuration Manager.
    pub fn subscribe_to_events(&self) -> ConfigurationEventReceiver {
        self.evt_sender.subscribe()
    }

    /// Creates a new command sender which can be used to send commands
    /// to the Configuration Manager.
    pub fn make_command_sender(&self) -> ConfigurationCommandSender {
        self.cmd_sender.clone()
    }
}

/// The set of commands that can be sent to the Configuration Manager.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigurationCommand {
    /// Starts the Configuration Manager. This will cause it to open
    /// or create the configuration database and load the data. Any
    /// automatic options such as updating the server list will be
    /// applied.
    Start,
    /// Stops the Configuration Manager. This will cause a complete
    /// stop of the program.
    Stop,
    /// Commands the Configuration Manager to update it server list.
    UpdateServerList,
}

/// The set of events that can be emitted by the Configuration Manager.
#[derive(Debug, Clone)]
pub enum ConfigurationEvents {
    InitComplete,
    SettingsChange(Arc<Settings>),
    AddressListChange(Arc<AddressList>),
    TempDirectoryListChange(Arc<TempDirectoryList>),
    ServerListChange(Arc<ServerList>),
}

/// This is private to the module: all access is via the handle.
pub struct ConfigurationManager {
    config_dir: PathBuf,
    config_db_filename: PathBuf,
    events_sender: ConfigurationEventSender,
    commands_receiver: ConfigurationCommandReceiver,
    conn: RefCell<Connection>,
    // Then the data.
    settings: Settings,
    addresses: AddressList,
    servers: ServerList,
    temp_dirs: TempDirectoryList,
}

impl ConfigurationManager {
    const CONFIG_DB_NAME: &str = "rmule_config.sqlite";

    /// Backs up the current configuration database.
    /// Deletes any out of date backups.
    pub fn backup(config_dir: &Path) -> Result<()> {
        let filename = Self::config_db_filename(config_dir);
        let backup_config_file = crate::file::make_backup_filename(&filename);

        if filename.try_exists()? {
            std::fs::copy(filename, &backup_config_file)?;
            info!(
                "Backed up config database to {}",
                backup_config_file.to_string_lossy()
            );
            let num_deleted = crate::file::delete_backups(config_dir, Self::CONFIG_DB_NAME, 10)?;
            info!(
                "Deleted {} backups of {}",
                num_deleted,
                Self::CONFIG_DB_NAME
            );
        } else {
            info!(
                "The configuration file {} does not exist, so it cannot be backed up",
                filename.to_string_lossy()
            );
        }

        Ok(())
    }

    /// Deletes the current configuration database.
    pub fn delete(config_dir: &Path) -> Result<()> {
        let filename = Self::config_db_filename(config_dir);
        file::delete_file_if_exists(&filename)
    }

    fn new<P>(
        events_sender: ConfigurationEventSender,
        commands_receiver: ConfigurationCommandReceiver,
        config_dir: P,
    ) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        let config_dir = config_dir.into();
        let config_db_filename = Self::config_db_filename(&config_dir);

        info!(
            "Attempting to open configuration database {}",
            config_db_filename.display()
        );

        // This will create an empty SQLite db if needed.
        let conn = Connection::open(&config_db_filename)?;
        migrations::apply_database_migrations(&conn)?;

        info!(
            "Opened configuration database {}",
            config_db_filename.display()
        );

        let settings = Settings::load(&conn)?;
        let addresses = AddressList::load_all(&conn)?;
        let servers = ServerList::load_all(&conn)?;
        let temp_dirs = TempDirectoryList::load_all(&conn)?;

        let cfg_mgr = Self {
            config_dir,
            config_db_filename,
            events_sender,
            commands_receiver,
            conn: RefCell::new(conn),
            settings,
            addresses,
            servers,
            temp_dirs,
        };

        Ok(cfg_mgr)
    }

    fn config_db_filename<P: Into<PathBuf>>(config_dir: P) -> PathBuf {
        let mut p = config_dir.into();
        p.push(Self::CONFIG_DB_NAME);
        p
    }

    /// Get the connection. Most ops can be performed on
    /// a shared connection.
    pub fn conn(&self) -> Ref<Connection> {
        self.conn.borrow()
    }

    /// Executes a transaction on this database.
    /// See [https://docs.rs/rusqlite/latest/rusqlite/struct.Transaction.html]
    pub fn execute_in_transaction<T, F>(
        &self,
        behaviour: TransactionBehavior,
        block: F,
    ) -> Result<T>
    where
        F: FnOnce(Transaction) -> Result<T>,
    {
        let mut conn = self.conn.borrow_mut();
        let txn = Transaction::new(&mut conn, behaviour)?;
        let result = block(txn)?;
        Ok(result)
    }

    fn run(&mut self) {
        while let Some(cmd) = self.commands_receiver.blocking_recv() {
            let shutdown = self.handle_message(cmd).unwrap();
            if shutdown {
                break;
            }
        }
    }

    /// Starts the ConfigurationManager loop. We wait for commands
    /// and then execute them, emitting events as necessary.
    fn handle_message(&mut self, cmd: ConfigurationCommand) -> Result<bool> {
        let mut shutdown = false;
        match cmd {
            ConfigurationCommand::Start => self.start()?,
            ConfigurationCommand::UpdateServerList => todo!(),
            ConfigurationCommand::Stop => {
                self.stop();
                shutdown = true;
            }
        }

        Ok(shutdown)
    }

    /// Starts the Configuration Manager. Everything is already loaded as
    /// that was done in `new`.
    fn start(&mut self) -> Result<()> {
        if self.settings.auto_update_server_list {
            let active_addresses: Vec<_> = self
                .addresses
                .iter()
                .filter_map(|addr| {
                    if addr.active {
                        Some(addr.url.clone())
                    } else {
                        None
                    }
                })
                .collect();

            if active_addresses.is_empty() {
                warn!("Cannot auto-update server list due to empty address table");
            } else {
                self.auto_update_server_list(&active_addresses)?;
            }
        }

        // Notify everybody of loaded data.
        // self.events_sender
        //     .send(ConfigurationEvents::SettingsChange(settings))?;
        // self.events_sender
        //     .send(ConfigurationEvents::AddressListChange(address_list))?;
        // self.events_sender
        //     .send(ConfigurationEvents::TempDirectoryListChange(temp_dirs))?;
        // self.events_sender
        //     .send(ConfigurationEvents::ServerListChange(servers))?;

        // // Tell everybody we are done with initial load.
        // self.events_sender.send(ConfigurationEvents::InitComplete)?;

        Ok(())
    }

    fn stop(&mut self) {}

    fn auto_update_server_list(&mut self, addresses: &Vec<String>) -> Result<()> {
        let download_servers = Self::download_servers(addresses)?;
        self.servers.merge_parsed_servers(&download_servers);
        let conn = self.conn.borrow();
        self.servers.save_all(&conn)?;
        Ok(())
    }

    fn download_servers(urls: &Vec<String>) -> Result<Vec<ParsedServer>> {
        info!("Downloading new servers");
        let mut tasks = Vec::new();

        for url in urls {
            let url = url.clone();
            tasks.push(tokio::spawn(async move {
                // If an eror occurs during download or parsing, do not abort the
                // program. Updating the server list is an "optional extra" and
                // we should not stop rMule from running because we got some
                // bad data from the internet.
                match Self::download_server_met(&url).await {
                    Ok(parsed_servers) => parsed_servers,
                    Err(_) => Vec::new(),
                }
            }));
        }

        // TODO: PR to document this!
        // Run some async code on the current runtime.
        let rt = tokio::runtime::Handle::current();
        let all_download_results = rt.block_on(join_all(tasks));

        let mut all_parsed_servers: Vec<_> = all_download_results
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect();

        all_parsed_servers.sort_by(|a, b| a.ip_addr.cmp(&b.ip_addr));
        all_parsed_servers.dedup_by(|a, b| a.ip_addr == b.ip_addr);
        info!("Retrieved {} unique servers", all_parsed_servers.len());
        Ok(all_parsed_servers)
    }

    async fn download_server_met(url: &str) -> Result<Vec<ParsedServer>> {
        info!("Downloading server.met from {}", url);

        let resp_bytes = reqwest::get(url)
            .await
            .with_context(|| format!("GET request to {} failed", url))?
            .bytes()
            .await
            .with_context(|| format!("Could not extract bytes from response from {}", url))?;

        let servers = if resp_bytes.is_empty() {
            Vec::new()
        } else {
            match parsing::parse_servers(&url, &resp_bytes) {
                Ok(parsed_servers) => parsed_servers,
                Err(e) => {
                    warn!("{}", e);
                    Vec::new()
                }
            }
        };

        info!(
            "Received {} bytes and {} servers from {}",
            resp_bytes.len(),
            servers.len(),
            url
        );

        Ok(servers)
    }
}
