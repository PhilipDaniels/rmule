use std::path::PathBuf;

use anyhow::{bail, Result};
use rmule::{file, get_default_config_dir, initialise_tokio_tracing, inititalise_config_dir};
use single_instance::SingleInstance;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    check_already_running()?;
    initialise_tokio_tracing();
    info!("Starting {}", env!("CARGO_PKG_NAME"));
    let parsed_args = parse_args()?;
    inititalise_config_dir(&parsed_args.config_directory, parsed_args.reset_config)?;
    info!("{} is waiting...", env!("CARGO_PKG_NAME"));
    //std::thread::sleep(Duration::from_secs(2));
    info!("Closing {}", env!("CARGO_PKG_NAME"));
    Ok(())
}

fn check_already_running() -> Result<()> {
    let instance = SingleInstance::new(env!("CARGO_PKG_NAME")).unwrap();
    if !instance.is_single() {
        bail!("{} is already running, only one instance can run at a time to prevent corruption of file downloads",
        env!("CARGO_PKG_NAME"));
    }

    Ok(())
}

fn parse_args() -> Result<ParsedArgs> {
    let mut args = pico_args::Arguments::from_env();
    let mut parsed_args = ParsedArgs::default();

    if args.contains("--help") {
        print_usage();
        std::process::exit(0);
    }

    // This argument allows the user to override the configuration directory
    // for each run. Handy for testing too.
    parsed_args.config_directory = match args.opt_value_from_str::<_, PathBuf>("--config-dir")? {
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
        match file::directory_exists(&parsed_args.config_directory)? {
            true => println!(
                "config dir: {} (exists)",
                parsed_args.config_directory.to_string_lossy()
            ),
            false => println!(
                "config dir:{} (does not exist, will be created)",
                parsed_args.config_directory.to_string_lossy()
            ),
        };
    }

    parsed_args.reset_config = args.contains("--reset-config");

    // If anything remains it means at least one invalid argument was passed.
    if !args.finish().is_empty() {
        print_usage();
        std::process::exit(0);
    }

    Ok(parsed_args)
}

#[rustfmt::skip]
fn print_usage() {
    eprintln!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("rmulegui [--help]                Print this message and exit");
    eprintln!("         [--config-dir DIR]      Specify a specific dir to read configuration from");
    eprintln!("         [--print-config-dir]    Print the effective configuration directory and exit");
    eprintln!("         [--reset-config]        Reset configuration to defaults");
}

#[derive(Default)]
struct ParsedArgs {
    config_directory: PathBuf,
    reset_config: bool,
}
