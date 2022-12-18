#![allow(dead_code)] // TEMP: Remove this when done!
#![forbid(unsafe_code)]

mod configuration;
pub mod file;
mod times;
mod utils;

use anyhow::{bail, Result};
use std::path::PathBuf;
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

pub fn inititalise() -> Result<()> {
    // This creates the directory, but no files within it.
    //file::ensure_directory_exists(&config_dir)?;

    // if reset_config() {
    //     ConfigurationDb::backup(&config_dir)?;
    //     // Deleting is enough, because we create a
    //     // new config db is one is not found.
    //     ConfigurationDb::delete(&config_dir)?;
    // }

    //let _cfg_mgr_handle = ConfigurationManagerHandle::new(&config_dir).await?;

    Ok(())
}
