use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Result, bail};
use config_dir::ConfigDir;
use mule_configuration::MuleConfiguration;
use single_instance::SingleInstance;

mod config_dir;
mod mule_configuration;
mod times;
mod file;

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
        None => ConfigDir::get_default_config_dir()?
    };

    let config_dir = ConfigDir::new(config_dir);

    // If this argument is specified, print the dir and then exit.
    if args.contains("--print-config-dir") {
        match file::directory_exists(config_dir.config_dir())? {
            true => println!("{} (exists)", config_dir.config_dir().to_string_lossy()),
            false => println!("{} (does not exist, will be created)", config_dir.config_dir().to_string_lossy()),
        };

        return Ok(());
    }

    // This creates the directory, but no files within it.
    file::ensure_directory_exists(config_dir.config_dir())?;
    
    if !config_dir.config_filename().try_exists()? {
        let new_config = MuleConfiguration::new(config_dir.config_dir());
        config_dir.save(&new_config)?;
    }

    if args.contains("--reset-config") {
        config_dir.backup_configuration()?;
        let new_config = MuleConfiguration::new(config_dir.config_dir());
        config_dir.save(&new_config)?;
    }

    // If anything remains it means at least one invalid argument was passed.
    if !args.finish().is_empty() {
        print_usage();
        return Ok(());
    }

    let mule_config = config_dir.load()?;
    file::ensure_directory_exists(&mule_config.temp_directory)?;
    file::ensure_directory_exists(&mule_config.incoming_directory)?;

    while true {
        std::thread::sleep(Duration::from_millis(100));
    }

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
