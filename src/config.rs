use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub key: String,
    pub action: String, // "project", "priority", "label"
    pub target: String, // project name, priority 1-5, or label name
}

impl QuickAction {
    pub fn get_description(&self) -> String {
        match self.action.as_str() {
            "project" => format!("Move to project: {}", self.target),
            "priority" => format!("Set priority to: {}", self.target),
            "label" => format!("Add label: {}", self.target),
            _ => format!("Unknown action: {} -> {}", self.action, self.target),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CriaConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub api_key_file: Option<String>,
    pub default_project: Option<String>,
    pub quick_actions: Option<Vec<QuickAction>>,
}

impl Default for CriaConfig {
    fn default() -> Self {
        CriaConfig {
            api_url: "https://vikunja.example.com/api/v1".to_string(),
            api_key: None,
            api_key_file: None,
            default_project: None,
            quick_actions: None,
        }
    }
}

impl CriaConfig {
    /// Load config from ~/.config/cria/config.yaml (XDG spec)
    #[allow(dead_code)]
    pub fn load() -> Option<Self> {
        Self::load_from_path(None)
    }

    /// Load config from a specific path, or default location if None
    pub fn load_from_path(custom_path: Option<&str>) -> Option<Self> {
        let config_path = if let Some(custom_path) = custom_path {
            PathBuf::from(custom_path)
        } else {
            // Use default XDG location
            match std::env::var("XDG_CONFIG_HOME") {
                Ok(val) => PathBuf::from(val).join("cria/config.yaml"),
                Err(_) => {
                    let mut home = dirs::home_dir()?;
                    home.push(".config/cria/config.yaml");
                    home
                }
            }
        };
        
        let contents = fs::read_to_string(&config_path).ok()?;
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

    /// Get quick actions as a HashMap keyed by the key character
    #[allow(dead_code)]
    pub fn get_quick_actions_map(&self) -> HashMap<String, QuickAction> {
        self.quick_actions
            .as_ref()
            .map(|actions| {
                actions
                    .iter()
                    .map(|action| (action.key.clone(), action.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a key has a quick action assigned
    #[allow(dead_code)]
    pub fn has_quick_action(&self, key: &str) -> bool {
        self.quick_actions
            .as_ref()
            .map(|actions| actions.iter().any(|action| action.key == key))
            .unwrap_or(false)
    }

    /// Get a quick action by key
    #[allow(dead_code)]
    pub fn get_quick_action(&self, key: &str) -> Option<&QuickAction> {
        self.quick_actions
            .as_ref()
            .and_then(|actions| actions.iter().find(|action| action.key == key))
    }
}
