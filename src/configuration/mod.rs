mod address;
mod configuration_db;
mod configuration_manager;
mod migrations;
mod settings;
mod sqlite_extensions;
mod temp_directory;

pub use address::*;
pub use configuration_db::*;
pub use settings::*;
pub use temp_directory::*;

use anyhow::Result;
use std::path::Path;
use tokio::sync::{broadcast, mpsc};

use self::configuration_manager::{ConfigurationCommands, ConfigurationEvents, ConfigurationManager};

pub type ConfigurationCommandSender = mpsc::Sender<ConfigurationCommands>;
pub type ConfigurationCommandReceiver = mpsc::Receiver<ConfigurationCommands>;

pub type ConfigurationEventSender = broadcast::Sender<ConfigurationEvents>;
pub type ConfigurationEventReceiver = broadcast::Receiver<ConfigurationEvents>;

/// Channel used by other components to send commands to the Configuration Service.
pub fn make_command_channel() -> (ConfigurationCommandSender, ConfigurationCommandReceiver) {
    mpsc::channel::<ConfigurationCommands>(32)
}

/// Channel used by the Configuration Service to send events out.
pub fn make_event_channel() -> (ConfigurationEventSender, ConfigurationEventReceiver) {
    broadcast::channel::<ConfigurationEvents>(32)
}

/// Initialises the Configuration Manager. This manager is responsible for
/// loading and saving information from and to the rmule_config.sqlite database.
/// It runs on its own Tokio task, receives commands via a channel
/// and emits events via a broadcast channel.
pub async fn initialise_configuration_manager(
    config_dir: &Path,
    events_sender: ConfigurationEventSender,
    command_receiver: ConfigurationCommandReceiver,
) -> Result<()> {
    let config_dir = config_dir.to_owned();

    tokio::spawn(async move {
        let mut mgr = ConfigurationManager::new(events_sender, command_receiver, config_dir);
        mgr.start().await?;
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}
