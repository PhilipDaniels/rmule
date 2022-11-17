use std::path::PathBuf;
use anyhow::Result;
use configuration::MuleConfiguration;

mod configuration;
//mod ini;
mod times;

fn main() -> Result<()> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains("--help") {
        print_usage();
        return Ok(());
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
        None => configuration::get_configuration_directory()?
    };

    // If this argument is specified, print the dir and then exit.
    if args.contains("--print-config-dir") {
        match configuration::configuration_directory_exists(&config_dir)? {
            true => println!("{} (exists)", config_dir.to_string_lossy()),
            false => println!("{} (does not exist, will be created)", config_dir.to_string_lossy()),
        }

        return Ok(());
    }

    if args.contains("--reset-config") {
        configuration::backup_configuration(&config_dir)?;
        configuration::save_configuration(&config_dir, &MuleConfiguration::default())?;
    }

    // If anything remains it means at least one invalid argument was passed.
    if !args.finish().is_empty() {
        print_usage();
        return Ok(());
    }

    configuration::ensure_configuration_directory_exists(&config_dir)?;
    let mule_config = configuration::load_configuration(&config_dir)?;
    
    // let mule_configuration = configuration::read_mule_configuration(&config_dir)?;
    // println!("app_version={:?}", mule_configuration.app_version());
    // println!("nickname={:?}", mule_configuration.nickname());
    // println!("confirm_exit={:?}", mule_configuration.confirm_exit());
    // println!("port={:?}", mule_configuration.port());
    Ok(())
}

fn print_usage() {
    eprintln!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    eprintln!("");
    eprintln!("rmule [--help]                Print this message and exit");
    eprintln!("      [--config-dir DIR]      Specify a specific dir to read configuration from");
    eprintln!("      [--print-config-dir]    Print the effective configuration directory and exit");
    eprintln!("      [--reset-config]        Reset configuration to defaults");
    eprintln!("      [--import-config DIR]   Import configuration from DIR (e.g. from aMule or eMule)");
}
