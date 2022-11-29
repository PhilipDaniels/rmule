use std::path::PathBuf;

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
    SettingsChange(Settings), // Try Arc<Settings> and remove Clone
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
        Self {
            config_dir: config_dir.into(),
            events_sender,
            commands_receiver,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        // Load things from the config db.
        let config_db = ConfigurationDb::open(&self.config_dir)?;
        self.load_all_configuration(&config_db)?;

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

    fn load_all_configuration(&mut self, config_db: &ConfigurationDb) -> Result<()> {
        let mut settings = Settings::load(&config_db)?;
        if settings.make_absolute(&config_db.config_dir) > 0 {
            settings.save(&config_db)?;
        }

        self.events_sender.send(ConfigurationEvents::SettingsChange(settings))?;

        let address_list = AddressList::load(&config_db)?;

        let mut temp_dirs = TempDirectoryList::load(&config_db)?;
        if temp_dirs.make_absolute(&config_db.config_dir) > 0 {
            temp_dirs.save(&config_db)?;
        }

        let mut num_added = temp_dirs.add("/phil/downloads")?;
        num_added += temp_dirs.add("/phil/downloads")?;
        num_added += temp_dirs.add("/foo/temp")?;
        if num_added > 0 {
            temp_dirs.save(&config_db)?;
        }
        Ok(())
    }
}