use std::{time::Duration, fs::{File, self}, path::Path, env::current_exe, ffi::OsString};
use anyhow::{Result, format_err};
use log::LevelFilter;
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    version: u8,
    pub log_level: LevelFilter,
    pub reg_cmd: String,
    #[serde(with = "humantime_serde")]
    pub exec_after_logon: Option<Duration>,
    #[serde(with = "humantime_serde")]
    pub exec_at_interval: Duration,
    pub exec_at_startup: bool,
}

impl Options {
    pub fn ensure() -> Result<Options> {
        let config_path = current_exe()?.with_file_name("config.json");

        if let Ok(existing_options) = Options::read_v1(&config_path) {
            Ok(existing_options)
        } else {
            let default_options: Options = serde_json::from_slice(include_bytes!("../config.json"))?;

            let file = File::create(config_path)?;
            serde_json::to_writer_pretty(file, &default_options)?;

            log::trace!("config::Options::ensure | Wrote config.json");
            Ok(default_options)
        }
    }

    fn read_v1(config_path: &Path) -> Result<Options> {
        let file = File::open(config_path)?;
        let options: Options = serde_json::from_reader(file)?;
    
        if options.version == 1 {
            log::trace!("config::read_v1 | Read config.json: {:?}", options);
            Ok(options)
        } else {
            Err(format_err!("Unknown config version"))
        }
    }
}

pub struct Payload {
    pub path: OsString
}

impl Payload {
    pub fn ensure() -> Result<Payload> {
        let payload_path = current_exe()?.with_file_name("config.reg");

        if !payload_path.exists() {
            fs::write(&payload_path, include_bytes!("../config.reg"))?;
        } 
        
        Ok(Payload { path: payload_path.into() } )
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use anyhow::Result;

    #[test]
    fn parse_options() -> Result<()> {
        let config_path_str = format!("config.json");
        let config_path = Path::new(&config_path_str);
        assert!(config_path.exists());
        super::Options::read_v1(&config_path)?;
        Ok(())
    }
}