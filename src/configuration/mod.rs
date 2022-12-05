mod address;
mod configuration_db;
mod configuration_manager;
mod migrations;
mod server;
mod settings;
mod sqlite_extensions;
mod temp_directory;

pub use address::*;
pub use configuration_db::*;
pub use configuration_manager::*;
pub use server::*;
pub use settings::*;
pub use temp_directory::*;

/*
/// Initialises the Configuration Manager. This manager is responsible for
/// loading and saving information from and to the rmule_config.sqlite database.
/// It runs on its own Tokio task, receives commands via a channel
/// and emits events via a broadcast channel.
pub async fn initialise_configuration_manager(
    config_dir: &Path,
) -> Result<ConfigurationManagerHandle> {
    let config_dir = config_dir.to_owned();

    tokio::task::Builder::new()
        .name("ConfigurationMgr")
        .spawn(async move {
            let mut mgr = ConfigurationManager::new(evt_tx, cmd_rx, config_dir);
            mgr.start().await?;
            Ok::<(), anyhow::Error>(())
        })?;
}
*/
