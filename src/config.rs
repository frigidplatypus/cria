use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct CriaConfig {
    pub api_url: String,
    pub api_key: String,
    pub default_project: Option<String>,
}

impl CriaConfig {
    /// Load config from ~/.config/cria/config.yaml (XDG spec)
    pub fn load() -> Option<Self> {
        let config_path = match std::env::var("XDG_CONFIG_HOME") {
            Ok(val) => PathBuf::from(val).join("cria/config.yaml"),
            Err(_) => {
                let mut home = dirs::home_dir()?;
                home.push(".config/cria/config.yaml");
                home
            }
        };
        let contents = fs::read_to_string(config_path).ok()?;
        serde_yaml::from_str(&contents).ok()
    }
}
