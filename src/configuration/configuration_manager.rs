use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{AddressList, ConfigurationDb, ServerList, Settings, TempDirectoryList};
use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{broadcast, mpsc};

pub type ConfigurationCommandSender = mpsc::Sender<ConfigurationCommands>;
pub type ConfigurationCommandReceiver = mpsc::Receiver<ConfigurationCommands>;

pub type ConfigurationEventSender = broadcast::Sender<ConfigurationEvents>;
pub type ConfigurationEventReceiver = broadcast::Receiver<ConfigurationEvents>;

/// The handle type allows commands to be sent and events to be received
/// from the Configuration Manager.
pub struct ConfigurationManagerHandle {
    cmd_sender: ConfigurationCommandSender,
    evt_sender: ConfigurationEventSender,
    // TODO: Without this we cannot emit events.
    evt_receiver: ConfigurationEventReceiver,
}

impl ConfigurationManagerHandle {
    pub fn new(config_dir: &Path) -> Result<Self> {
        let (cmd_sender, cmd_receiver) = Self::make_command_channel();
        let (evt_sender, evt_receiver) = Self::make_event_channel();
        let evt_sender2 = evt_sender.clone();

        let handle = ConfigurationManagerHandle {
            cmd_sender,
            evt_sender,
            evt_receiver,
        };

        // Construct a new ConfigurationManager to manage the connection
        // to the configuration database.
        let mut mgr = ConfigurationManager::new(evt_sender2, cmd_receiver, config_dir);
        mgr.load_all_configuration()?;

        // Spawn a new task to make the ConfigurationManager respond to events.
        tokio::task::Builder::new()
            .name("ConfigurationMgr")
            .spawn(async move {
                mgr.start().await.expect("Start never fails");
            })?;

        Ok(handle)
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

    /// Channel used by other components to send commands to the Configuration
    /// Manager.
    fn make_command_channel() -> (ConfigurationCommandSender, ConfigurationCommandReceiver) {
        mpsc::channel::<ConfigurationCommands>(32)
    }

    /// Channel used by the Configuration Manager to emit events.
    fn make_event_channel() -> (ConfigurationEventSender, ConfigurationEventReceiver) {
        broadcast::channel::<ConfigurationEvents>(32)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigurationCommands {
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum ConfigurationEvents {
    InitComplete,
    SettingsChange(Arc<Settings>),
    AddressListChange(Arc<AddressList>),
    TempDirectoryListChange(Arc<TempDirectoryList>),
    ServerListChange(Arc<ServerList>),
}

pub struct ConfigurationManager {
    config_dir: PathBuf,
    events_sender: Sender<ConfigurationEvents>,
    commands_receiver: Receiver<ConfigurationCommands>,
    config_db: Option<ConfigurationDb>,
}

impl ConfigurationManager {
    pub fn new<P>(
        events_sender: Sender<ConfigurationEvents>,
        commands_receiver: Receiver<ConfigurationCommands>,
        config_dir: P,
    ) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            config_dir: config_dir.into(),
            events_sender,
            commands_receiver,
            config_db: None,
        }
    }

    pub fn load_all_configuration(&mut self) -> Result<()> {
        let config_db = ConfigurationDb::open(&self.config_dir)?;

        self.load_settings(&config_db)?;
        self.load_address_list(&config_db)?;
        self.load_temp_directories(&config_db)?;
        self.load_servers(&config_db)?;

        self.config_db = Some(config_db);

        // Tell everybody we are ready.
        self.events_sender.send(ConfigurationEvents::InitComplete)?;

        Ok(())
    }

    /// Starts the ConfigurationManager loop. We wait for commands
    /// and then execute them, emitting events as necessary.
    pub async fn start(&mut self) -> Result<()> {
        while let Some(cmd) = self.commands_receiver.recv().await {
            if cmd == ConfigurationCommands::Shutdown {
                self.shutdown();
                break;
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn load_settings(&self, config_db: &ConfigurationDb) -> Result<(), anyhow::Error> {
        let settings = Settings::load(config_db)?;
        self.events_sender
            .send(ConfigurationEvents::SettingsChange(Arc::new(settings)))?;
        Ok(())
    }

    fn load_address_list(&self, config_db: &ConfigurationDb) -> Result<(), anyhow::Error> {
        let address_list = AddressList::load_all(config_db)?;
        self.events_sender
            .send(ConfigurationEvents::AddressListChange(Arc::new(
                address_list,
            )))?;
        Ok(())
    }

    fn load_temp_directories(&self, config_db: &ConfigurationDb) -> Result<()> {
        let temp_dirs = TempDirectoryList::load_all(config_db)?;
        self.events_sender
            .send(ConfigurationEvents::TempDirectoryListChange(Arc::new(
                temp_dirs,
            )))?;
        Ok(())
    }

    fn load_servers(&self, config_db: &ConfigurationDb) -> Result<()> {
        let servers = ServerList::load_all(config_db)?;
        self.events_sender
            .send(ConfigurationEvents::ServerListChange(Arc::new(servers)))?;
        Ok(())
    }
}
