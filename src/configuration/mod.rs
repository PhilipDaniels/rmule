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

/// Initialises the Configuration Manager. This manager is responsible for
/// loading and saving information from and to the rmule_config.sqlite database.
/// It runs on its own Tokio task, receives commands via a channel
/// and emits events via a broadcast channel.
pub async fn initialise_configuration_manager(
    config_dir: &Path,
) -> Result<(ConfigurationCommandSender, ConfigurationEventReceiver)> {
    let config_dir = config_dir.to_owned();

    let (cmd_tx, cmd_rx) = make_command_channel();
    let (evt_tx, evt_rx) = make_event_channel();

    tokio::spawn(async move {
        let mut mgr = ConfigurationManager::new(evt_tx, cmd_rx, config_dir);
        mgr.start().await?;
        Ok::<(), anyhow::Error>(())
    });

    Ok((cmd_tx, evt_rx))
}

/// Channel used by other components to send commands to the Configuration
/// Service.
fn make_command_channel() -> (ConfigurationCommandSender, ConfigurationCommandReceiver) {
    mpsc::channel::<ConfigurationCommands>(32)
}

/// Channel used by the Configuration Service to send events out.
fn make_event_channel() -> (ConfigurationEventSender, ConfigurationEventReceiver) {
    broadcast::channel::<ConfigurationEvents>(32)
}
