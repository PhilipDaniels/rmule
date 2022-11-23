use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// Holds the main configuration settings for rMule.
/// Most of these fields are pulled from rmule.conf.
#[derive(Serialize, Deserialize, Debug)]
pub struct MuleConfiguration {
    nickname: String,
    /// Directory in which temporary (currently downloading) files are to be stored.
    pub temp_directory: PathBuf,
    /// Directory in which files are to be placed when they are finished downloading.
    pub incoming_directory: PathBuf
}

impl MuleConfiguration {
    /// Creates new, default Mule configuration.
    pub fn new(config_dir: &Path) -> Self {
        let mut temp_pb = config_dir.to_owned();
        temp_pb.push("temp");

        let mut incoming_pb = config_dir.to_owned();
        incoming_pb.push("downloaded");

        let c = Self {
            nickname: "http://www.aMule.org".to_owned(),
            temp_directory: temp_pb,
            incoming_directory: incoming_pb
        };

        c
    }
}

