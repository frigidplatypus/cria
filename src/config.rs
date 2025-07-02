use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf; 

#[derive(Debug, Serialize, Deserialize)]
pub struct CriaConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub api_key_file: Option<String>,
    pub default_project: Option<String>,
}

impl Default for CriaConfig {
    fn default() -> Self {
        CriaConfig {
            api_url: "https://vikunja.example.com/api/v1".to_string(),
            api_key: None,
            api_key_file: None,
            default_project: None,
        }
    }
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

    /// Check if any API key configuration is present
    pub fn has_api_key_config(&self) -> bool {
        (self.api_key.is_some() && !self.api_key.as_ref().unwrap().trim().is_empty()) ||
        (self.api_key_file.is_some() && !self.api_key_file.as_ref().unwrap().trim().is_empty())
    }

    /// Get the API key, preferring api_key over api_key_file
    pub fn get_api_key(&self) -> Result<String, String> {
        // First try direct api_key
        if let Some(ref key) = self.api_key {
            if !key.trim().is_empty() {
                return Ok(key.clone());
            }
        }

        // Fall back to api_key_file
        if let Some(ref key_file) = self.api_key_file {
            match self.read_api_key_from_file(key_file) {
                Ok(key) => return Ok(key),
                Err(e) => {
                    return Err(format!(
                        "Failed to read API key from file '{}': {}",
                        key_file, e
                    ))
                }
            }
        }

        Err("No API key found. Please set either 'api_key' or 'api_key_file' in config.yaml".to_string())
    }

    /// Read API key from file
    fn read_api_key_from_file(&self, file_path: &str) -> Result<String, std::io::Error> {
        let path = if file_path.starts_with("~/") {
            // Expand tilde to home directory
            let home = dirs::home_dir().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")
            })?;
            home.join(&file_path[2..])
        } else if file_path.starts_with('/') {
            // Absolute path
            PathBuf::from(file_path)
        } else {
            // Relative path - relative to config directory
            let config_dir = match std::env::var("XDG_CONFIG_HOME") {
                Ok(val) => PathBuf::from(val).join("cria"),
                Err(_) => {
                    let mut home = dirs::home_dir().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")
                    })?;
                    home.push(".config/cria");
                    home
                }
            };
            config_dir.join(file_path)
        };

        let contents = fs::read_to_string(path)?;
        Ok(contents.trim().to_string())
    }
}
