#![allow(dead_code)] // TEMP: Remove this when done!
#![forbid(unsafe_code)]

use anyhow::{bail, Result};
use configuration::ConfigurationDb;
use single_instance::SingleInstance;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::configuration::ConfigurationManagerHandle;

mod configuration;
mod file;
mod parsers;
mod times;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise the Tokio tracing system.
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::INFO)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed");

    info!("STARTING RMULE");

    let mut args = pico_args::Arguments::from_env();

    if args.contains("--help") {
        print_usage();
        return Ok(());
    }

    let instance = SingleInstance::new("rmule").unwrap();
    if !instance.is_single() {
        bail!("rmule is already running, only one instance can run at a time to prevent corruption of file downloads");
    }

    // This argument allows the user to override the configuration directory
    // for each run. Handy for testing too.
    let config_dir = match args.opt_value_from_str::<_, PathBuf>("--config-dir")? {
        Some(override_dir) => {
            if override_dir.is_relative() {
                let mut cwd = std::env::current_dir()?;
                cwd.push(override_dir);
                cwd
            } else {
                override_dir
            }
        }
        None => get_default_config_dir()?,
    };

    if args.contains("--print-config-dir") {
        match file::directory_exists(&config_dir)? {
            true => println!("{} (exists)", config_dir.to_string_lossy()),
            false => println!(
                "{} (does not exist, will be created)",
                config_dir.to_string_lossy()
            ),
        };

        return Ok(());
    }

    // This creates the directory, but no files within it.
    file::ensure_directory_exists(&config_dir)?;

    if args.contains("--reset-config") {
        ConfigurationDb::backup(&config_dir)?;
        // Deleting is enough, because we create a
        // new config db is one is not found.
        ConfigurationDb::delete(&config_dir)?;
    }

    // If anything remains it means at least one invalid argument was passed.
    if !args.finish().is_empty() {
        print_usage();
        return Ok(());
    }

    let _cfg_mgr_handle = ConfigurationManagerHandle::new(&config_dir).await?;

    // Without this, the process will exit before the Configuration Manager
    // background task has had chance to run and load all data.
    info!("RMULE IS WAITING...");
    std::thread::sleep(Duration::from_secs(5));

    info!("CLOSING RMULE");

    Ok(())
}

fn print_usage() {
    eprintln!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("rmule [--help]                Print this message and exit");
    eprintln!("      [--config-dir DIR]      Specify a specific dir to read configuration from");
    eprintln!("      [--print-config-dir]    Print the effective configuration directory and exit");
    eprintln!("      [--reset-config]        Reset configuration to defaults");
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
