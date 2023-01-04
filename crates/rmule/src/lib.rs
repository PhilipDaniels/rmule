#![allow(dead_code)] // TEMP: Remove this when done!
#![forbid(unsafe_code)]

pub mod configuration;
mod engine;
pub mod file;
mod times;
mod utils;

use anyhow::{bail, Result};
use configuration::ConfigurationManager;
pub use engine::Engine;
use std::path::{Path, PathBuf};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Initialise the Tokio tracing system.
pub fn initialise_tokio_tracing() {
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");
}

/// Finds the directory that holds all the configuration information
/// for rMule. We have our own directory, separate from aMule/eMule.
/// The directory may not exist (and may even be a file on disk, this
/// function does not check any of that, it just creates a PathBuf
/// with the correct path).
pub fn get_default_config_dir() -> Result<PathBuf> {
    let mut cfg_dir = match dirs::config_dir() {
        Some(pb) => pb,
        None => bail!("Cannot determine home directory"),
    };

    cfg_dir.push("rMule");
    Ok(cfg_dir)
}

/// Initialises the configuration directory - simply we make sure it exists
/// and optionally reset the configuration by deleting the database - a new
/// default configuration db will be created at startup of the actor system.
pub fn inititalise_config_dir(config_dir: &Path, reset: bool) -> Result<()> {
    // This creates the directory, but no files within it.
    file::ensure_directory_exists(config_dir)?;

    if reset {
        ConfigurationManager::backup(config_dir)?;
        // Deleting is enough, because we create a
        // new config db if one is not found.
        ConfigurationManager::delete(config_dir)?;
    }

    Ok(())
}

/// Creates a new rMule Engine. The engine is not yet running,
/// it must be started before it will respond to commands.
pub async fn initialise_engine(
    config_dir: &Path,
    tokio_handle: tokio::runtime::Handle,
) -> Result<Engine> {
    let engine = Engine::new(config_dir, tokio_handle);
    Ok(engine)
}
