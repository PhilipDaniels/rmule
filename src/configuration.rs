use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::ini::IniFile;

const CONFIG_DIR: &str  = ".rMule";
const CONFIG_FILENAME: &str = "rmule.conf";

/// Finds the directory that holds all the configuration information
/// for rMule. We have our own directory, separate from aMule/eMule.
/// The directory may not exist (and may even be a file).
pub fn get_configuration_directory() -> Result<PathBuf> {
    let mut cfg_dir = match dirs::config_dir() {
        Some(pb) => pb,
        None => bail!("Cannot determine home directory"),
    };

    cfg_dir.push(CONFIG_DIR);
    Ok(cfg_dir)
}

/// Check to see if the configuration directory exists.
pub fn configuration_dir_exists(config_dir: &Path) -> Result<bool> {
    Ok(config_dir.try_exists()?)
}

/// Creates the configuration directory if it does not exist. If it already
/// exists, then it is checked to ensure that it is a directory and not a file
/// or a symlink.
pub fn ensure_configuration_directory_exists(config_dir: &Path) -> Result<()> {
    if !configuration_dir_exists(config_dir)? {
        std::fs::create_dir_all(&config_dir)?;
    } else {
        if !config_dir.is_dir() {
            bail!("Configuration directory {} is not a directory", config_dir.to_string_lossy());
        }
    }

    Ok(())
}





pub struct MuleConfiguration {
    raw_lines: Vec<String>,
    ini_data: IniFile,
}

impl MuleConfiguration {
    pub fn app_version(&self) -> &str {
        self.ini_data.get_str("eMule", "AppVersion")
    }

    pub fn nickname(&self) -> &str {
        self.ini_data.get_str("eMule", "Nick")
    }

    pub fn confirm_exit(&self) -> bool {
        self.ini_data.get_bool("eMule", "ConfirmExit")
    }

    pub fn port(&self) -> u16 {
        self.ini_data.get_i32("eMule", "Port").try_into().unwrap()
    }
}

pub fn read_mule_configuration(config_dir: &Path) -> Result<MuleConfiguration> {
    let mut pb = config_dir.to_path_buf();
    pb.push("amule.conf");
    let f = File::open(&pb)?;
    let reader = BufReader::new(f);
    let raw_lines = reader.lines().collect::<std::io::Result<Vec<String>>>()?;
    let parsed_lines = IniFile::new(raw_lines.clone());

    Ok(MuleConfiguration {
        raw_lines,
        ini_data: parsed_lines,
    })
}


