use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc::Receiver;

use super::{AddressList, ConfigurationDb, Settings, TempDirectoryList};

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
}

pub struct ConfigurationManager {
    config_dir: PathBuf,
    events_sender: Sender<ConfigurationEvents>,
    commands_receiver: Receiver<ConfigurationCommands>,
}

impl ConfigurationManager {
    pub fn new(
        events_sender: Sender<ConfigurationEvents>,
        commands_receiver: Receiver<ConfigurationCommands>,
        config_dir: PathBuf,
    ) -> Self {
        Self { config_dir: config_dir.into(), events_sender, commands_receiver }
    }

    pub async fn start(&mut self) -> Result<()> {
        // Load things from the config db.
        let config_db = ConfigurationDb::open(&self.config_dir)?;
        self.load_settings(&config_db)?;
        self.load_address_list(&config_db)?;
        self.load_temp_directories(&config_db)?;

        // Tell everybody we are ready.
        self.events_sender.send(ConfigurationEvents::InitComplete)?;

        // Wait for work and despatch it.
        while let Some(cmd) = self.commands_receiver.recv().await {
            if cmd == ConfigurationCommands::Shutdown {
                self.shutdown();
                break;
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {}

    fn load_address_list(&mut self, config_db: &ConfigurationDb) -> Result<(), anyhow::Error> {
        let address_list = AddressList::load_all(&config_db)?;
        self.events_sender.send(ConfigurationEvents::AddressListChange(Arc::new(address_list)))?;
        Ok(())
    }

    fn load_settings(&mut self, config_db: &ConfigurationDb) -> Result<(), anyhow::Error> {
        let settings = Settings::load(&config_db)?;
        self.events_sender.send(ConfigurationEvents::SettingsChange(Arc::new(settings)))?;
        Ok(())
    }

    fn load_temp_directories(&mut self, config_db: &ConfigurationDb) -> Result<()> {
        let temp_dirs = TempDirectoryList::load_all(&config_db)?;
        self.events_sender
            .send(ConfigurationEvents::TempDirectoryListChange(Arc::new(temp_dirs)))?;
        Ok(())
    }
}
