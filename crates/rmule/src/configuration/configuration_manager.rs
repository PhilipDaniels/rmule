use super::{Address, AddressList, ConfigurationDb, ServerList, Settings, TempDirectoryList};
use crate::configuration::parsing::{self, ParsedServer};
use anyhow::{Context, Result};
use futures::future::join_all;
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
        let mut mgr = ConfigurationManager::new(evt_sender.clone(), cmd_receiver, config_dir);

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
struct ConfigurationManager {
    config_dir: PathBuf,
    events_sender: ConfigurationEventSender,
    commands_receiver: ConfigurationCommandReceiver,
}

impl ConfigurationManager {
    fn new<P>(
        events_sender: ConfigurationEventSender,
        commands_receiver: ConfigurationCommandReceiver,
        config_dir: P,
    ) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            config_dir: config_dir.into(),
            events_sender,
            commands_receiver,
        }
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
            ConfigurationCommand::Start => self.load_all_configuration()?,
            ConfigurationCommand::UpdateServerList => todo!(),
            ConfigurationCommand::Stop => {
                self.stop();
                shutdown = true;
            }
        }

        Ok(shutdown)
    }

    fn load_all_configuration(&mut self) -> Result<()> {
        let config_db = ConfigurationDb::open(&self.config_dir)?;
        let conn = config_db.conn();

        let settings = Arc::new(Settings::load(&conn)?);
        let address_list = Arc::new(AddressList::load_all(&conn)?);

        let servers = if settings.auto_update_server_list {
            let active_addresses: Vec<_> = address_list
                .iter()
                .filter(|addr| addr.active == true)
                .collect();

            if active_addresses.is_empty() {
                warn!("Cannot auto-update server list due to empty address table");
                Arc::new(ServerList::load_all(&config_db)?)
            } else {
                self.auto_update_server_list(&config_db, &active_addresses)?
            }
        } else {
            Arc::new(ServerList::load_all(&config_db)?)
        };

        let temp_dirs = Arc::new(TempDirectoryList::load_all(&config_db)?);

        // Store this for later use.
        //self.config_db = Some(config_db);

        // Notify everybody of loaded data.
        self.events_sender
            .send(ConfigurationEvents::SettingsChange(settings))?;
        self.events_sender
            .send(ConfigurationEvents::AddressListChange(address_list))?;
        self.events_sender
            .send(ConfigurationEvents::TempDirectoryListChange(temp_dirs))?;
        self.events_sender
            .send(ConfigurationEvents::ServerListChange(servers))?;

        // Tell everybody we are done with initial load.
        self.events_sender.send(ConfigurationEvents::InitComplete)?;

        Ok(())
    }

    fn stop(&mut self) {}

    fn auto_update_server_list(
        &self,
        config_db: &ConfigurationDb,
        addresses: &[&Address],
    ) -> Result<Arc<ServerList>> {
        info!("Auto-updating server list");

        let mut current_servers = ServerList::load_all(config_db)?;

        let mut tasks = Vec::new();

        for addr in addresses {
            let url = addr.url.clone();
            tasks.push(tokio::spawn(async move {
                // If an eror occurs during download or parsing, do not abort the
                // program. Updating the server list is an "optional extra" and we
                // should not stop rMule from running because we got some bad data
                // from the internet.
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

        current_servers.merge_parsed_servers(&all_parsed_servers);
        ServerList::save_all(&mut current_servers, config_db)?;
        Ok(Arc::new(current_servers))
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
