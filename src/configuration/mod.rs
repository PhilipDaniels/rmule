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

use self::configuration_manager::{
    ConfigurationCommands, ConfigurationEvents, ConfigurationManager,
};

pub type ConfigurationCommandSender = mpsc::Sender<ConfigurationCommands>;
pub type ConfigurationEventReceiver = broadcast::Receiver<ConfigurationEvents>;

/// Initialises the Configuration Manager. This manager is responsible for
/// loading and saving information from and to the rmule_config.sqlite database.
/// It runs on its own Tokio task, receives commands via a channel
/// and emits events via a broadcast channel.
pub fn initialise_configuration_manager(
    config_dir: &Path,
) -> Result<(ConfigurationCommandSender, ConfigurationEventReceiver)> {
    // Channel used by other components to send commands to the Configuration Service.
    let (config_commands_tx, config_commands_rx) = mpsc::channel::<ConfigurationCommands>(32);

    // Channel used by the Configuration Service to send events out.
    let (config_events_tx, config_events_rx) = broadcast::channel::<ConfigurationEvents>(32);

    let config_dir = config_dir.to_owned();

    // The Configuration Service takes ownership of config_commands_rx and config_events_tx
    tokio::spawn(async move {
        let mut svc = ConfigurationManager::new(config_events_tx, config_commands_rx, config_dir);
        svc.start().await?;
        Ok::<(), anyhow::Error>(())
    });

    Ok((config_commands_tx, config_events_rx))
}
