use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Result, bail};
use configuration::{ConfigurationDb, Settings};
use single_instance::SingleInstance;

//mod config_dir;
//mod mule_configuration;
mod times;
mod file;
//mod server;

mod configuration;

fn main() -> Result<()> {
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
        },
        None => get_default_config_dir()?
    };

    // If this argument is specified, print the dir and then exit.
    if args.contains("--print-config-dir") {
        match file::directory_exists(&config_dir)? {
            true => println!("{} (exists)", config_dir.to_string_lossy()),
            false => println!("{} (does not exist, will be created)", config_dir.to_string_lossy()),
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

    let config_db = ConfigurationDb::open(&config_dir)?;
    
    // If anything remains it means at least one invalid argument was passed.
    if !args.finish().is_empty() {
        print_usage();
        return Ok(());
    }

    let settings = Settings::load(&config_db)?;
    file::ensure_directory_exists(&settings.downloaded_directory)?;
    eprintln!("Settings = {:?}", settings);

    // let server_list = ServerList::load(config_dir.server_filename())?;

    Ok(())
}

fn print_usage() {
    eprintln!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("rmule [--help]                Print this message and exit");
    eprintln!("      [--config-dir DIR]      Specify a specific dir to read configuration from");
    eprintln!("      [--print-config-dir]    Print the effective configuration directory and exit");
    eprintln!("      [--reset-config]        Reset configuration to defaults");
    eprintln!("      [--import-config DIR]   Import configuration from DIR (e.g. from aMule or eMule)");
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
