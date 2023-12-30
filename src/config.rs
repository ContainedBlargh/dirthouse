use std::fs::File;
use std::io::Read;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub descriptor: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirtConfig {
    pub serve_dir: String,
    pub host_addr: String,
    pub port: u16,
    pub additional_packages: Vec<Package>
}

pub fn load() -> DirtConfig {
    fn try_load() -> serde_json::error::Result<DirtConfig> {
        let mut file = File::open("config.json").map_err(serde::de::Error::custom)?;
        let mut data = String::new();
        file.read_to_string(&mut data).map_err(serde::de::Error::custom)?;
        serde_json::from_str(&data)
    }

    try_load().unwrap_or(DirtConfig {
        serve_dir: "dist".to_string(),
        host_addr: "127.0.0.1".to_string(),
        port: 7642,
        additional_packages: vec![]
    })
}
