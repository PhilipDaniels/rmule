#![allow(dead_code)] // TEMP: Remove this when done!
#![forbid(unsafe_code)]

use anyhow::{bail, Result};
use rmule::{
    create_engine, file, get_default_config_dir, initialise_tokio_tracing, inititalise_config_dir,
};
use single_instance::SingleInstance;
use std::path::PathBuf;
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::info;

mod ui;
mod widgets;

fn main() -> Result<()> {
    check_already_running()?;
    let parsed_args = parse_args()?;

    let rt = Runtime::new().expect("Unable to create Tokio Runtime");

    // The handle can be cloned and passed into other
    // threads, we will need this to spawn async tasks
    // from non-tokio threads, such as those which are
    // running dedicated actors.
    let tokio_handle = rt.handle().clone();

    // Execute the runtime in its own thread, thus reserving the
    // main thread for egui.
    // The future doesn't have to do anything.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    initialise_tokio_tracing();

    info!("Starting {}", env!("CARGO_PKG_NAME"));

    inititalise_config_dir(&parsed_args.config_directory, parsed_args.reset_config)?;

    let engine = create_engine(&parsed_args.config_directory, tokio_handle)?;
    ui::show_main_window(engine);

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
