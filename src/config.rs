use std::fs::File;
use std::io::Read;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DirtConfig {
    pub serve_dir: String,
    pub host_addr: String,
    pub port: u16,
}

pub fn load() -> DirtConfig {
    fn try_load() -> serde_json::error::Result<DirtConfig> {
        let mut file = File::open("config.json").map_err(|e| serde::de::Error::custom(e))?;
        let mut data = String::new();
        file.read_to_string(&mut data).map_err(|e| serde::de::Error::custom(e))?;
        return serde_json::from_str(&data);
    }

    try_load().unwrap_or(DirtConfig {
        serve_dir: "dist".to_string(),
        host_addr: "127.0.0.1".to_string(),
        port: 8080,
    })
}
