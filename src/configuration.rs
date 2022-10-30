use anyhow::{Result, bail};
use serde::{Deserialize, Deserializer};
use std::path::{PathBuf, Path};
use std::str::FromStr;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MuleConfiguration {
    #[serde(deserialize_with = "de_string")]
    app_version: String,
    #[serde(rename = "Nick")]
    nickname: String,
    confirm_exit: bool
}

pub fn read_mule_configuration(config_dir: &Path) -> Result<MuleConfiguration> {
    let mut pb = config_dir.to_path_buf();
    pb.push("amule.conf");
    let file_contents = std::fs::read_to_string(&pb)?;
    let conf = toml_edit::de::from_str(&file_contents)?;
    Ok(conf)
}

/// Finds the directory that holds all the configuration information.
/// For now, very simple only works on my Linux :-)
pub fn get_configuration_directory() -> Result<PathBuf> {
    let home_dir = match dirs::home_dir() {
        Some(pb) => pb,
        None => bail!("Cannot determine home directory")
    };

    let mut hd = home_dir.clone();
    hd.push(".aMule");
    if !hd.is_dir() {
        bail!("Expected home directory {} does not exist", hd.display());
    }

    Ok(hd)
}
