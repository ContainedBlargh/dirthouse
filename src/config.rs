use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub options: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirtConfig {
    pub app_name: String,
    pub serve_dir: String,
    pub host_addr: String,
    pub port: u16,
    pub additional_packages: Option<Vec<Package>>,
    pub cleanup: Option<bool>,
}

pub fn load(path: String) -> DirtConfig {
    fn try_load(path: String) -> serde_json::error::Result<DirtConfig> {
        let mut file = File::open(path).map_err(serde::de::Error::custom)?;
        let mut data: Vec<_> = Vec::new();
        file.read_to_end(&mut data).map_err(serde::de::Error::custom)?;
        serde_json::from_slice(&data)
    }

    try_load(path).unwrap_or_else(|err| {
        eprintln!(
            "Warning: Could not parse JSON config, either provide the path to one as an argument \
            or create a config.json file in this directory."
        );
        eprintln!("{:?}", err.to_string());
        DirtConfig {
            app_name: "app".to_string(),
            serve_dir: "dist".to_string(),
            host_addr: "127.0.0.1".to_string(),
            port: 7642,
            additional_packages: None,
            cleanup: None,
        }
    })
}
