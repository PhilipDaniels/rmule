use std::path::{PathBuf, Path};
use anyhow::{bail, Result};

use crate::mule_configuration::MuleConfiguration;
use crate::times;

pub struct ConfigDir {
    base_dir: PathBuf,
    config_filename: PathBuf,
}

impl ConfigDir {
    const CONFIG_DIR: &str  = "rMule";
    const CONFIG_FILENAME: &str = "rmule.conf";
    const CONFIG_BACKUP_PREFIX: &str = "rmule.conf-";

    pub fn new<P: Into<PathBuf>>(dir: P) -> Self {
        let dir = dir.into();
        let base_dir = dir.clone();
        let mut config_filename = dir.clone();
        config_filename.push(Self::CONFIG_FILENAME);

        Self {
            base_dir,
            config_filename
        }
    }

    pub fn config_dir(&self) -> &Path {
        &self.base_dir
    }

    pub fn config_filename(&self) -> &Path {
        &self.config_filename
    }

    /// Backs up the configuration file, if it exists. If it does not exist
    /// then the function does nothing.
    pub fn backup_configuration(&self) -> Result<()> {
        if self.config_filename.try_exists()? {
            let mut backup_config_file = self.base_dir.clone();
            backup_config_file.push(format!("{}{}", Self::CONFIG_BACKUP_PREFIX, times::current_date_to_yyyy_mm_dd()));
            std::fs::copy(&self.config_filename, backup_config_file)?;
            self.delete_old_configuration_backups()?;
        }

        Ok(())
    }

    /// Deletes all the old backup files - we keep a maximum of 10.
    fn delete_old_configuration_backups(&self) -> Result<()> {
        let mut backups_to_delete = Vec::new();

        for entry in self.base_dir.read_dir()? {
            let path = entry?.path();
            if path.is_file() {
                if let Some(fname) = path.file_name() {
                    let fname = fname.to_string_lossy();
                    if fname.starts_with(Self::CONFIG_BACKUP_PREFIX) {
                        backups_to_delete.push(path);
                    }
                }
            }
        }
    
        backups_to_delete.sort();
    
        for backup in backups_to_delete.into_iter().rev().skip(10) {
            std::fs::remove_file(backup)?;
        }
    
        Ok(())
    }

    /// Saves the configuration to the configuration file.
    pub fn save(&self, mule_config: &MuleConfiguration) -> Result<()> {
        let toml = toml::to_string_pretty(mule_config)?;
        std::fs::write(self.config_filename(), &toml)?;
        Ok(())
    }

    /// Loads the configuration from the configuration file.
    pub fn load(&self) -> Result<MuleConfiguration> {
        let config_file_contents = std::fs::read_to_string(&self.config_filename)?;
        let config = toml::from_str(&config_file_contents)?;
        Ok(config)
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
    
        cfg_dir.push(Self::CONFIG_DIR);
        Ok(cfg_dir)
    }
}
