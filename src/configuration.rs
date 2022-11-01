use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::ini::IniFile;

pub struct MuleConfiguration {
    raw_lines: Vec<String>,
    parsed_lines: IniFile,
}

impl MuleConfiguration {
    fn get_str(&self, section: &str, key: &str) -> &str {
        self.get_str_opt(section, key).unwrap()
    }

    fn get_str_opt(&self, section: &str, key: &str) -> Option<&str> {
        self.parsed_lines
            .get_entry_in_section(section, key)
            .and_then(|e| Some(e.get_value()))
    }

    fn get_bool(&self, section: &str, key: &str) -> bool {
        self.get_bool_opt(section, key).unwrap()
    }

    fn get_bool_opt(&self, section: &str, key: &str) -> Option<bool> {
        match self.get_str_opt(section, key) {
            Some("1") => Some(true),
            Some(_) => Some(false),
            None => None,
        }
    }

    pub fn app_version(&self) -> &str {
        self.get_str("eMule", "AppVersion")
    }

    pub fn nickname(&self) -> &str {
        self.get_str("eMule", "Nick")
    }

    pub fn confirm_exit(&self) -> bool {
        self.get_bool("eMule", "ConfirmExit")
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
        parsed_lines,
    })
}

/// Finds the directory that holds all the configuration information.
/// For now, very simple only works on my Linux :-)
pub fn get_configuration_directory() -> Result<PathBuf> {
    let home_dir = match dirs::home_dir() {
        Some(pb) => pb,
        None => bail!("Cannot determine home directory"),
    };

    let mut hd = home_dir.clone();
    hd.push(".aMule");
    if !hd.is_dir() {
        bail!("Expected home directory {} does not exist", hd.display());
    }

    Ok(hd)
}
