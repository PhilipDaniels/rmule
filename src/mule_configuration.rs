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


/*
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
*/
