use crate::config::CriaConfig;
use std::io::{self, Write};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

pub fn first_run_wizard() -> Option<CriaConfig> {
    println!("Welcome to Cria first run setup!");
    let mut base_url = String::new();
    let mut default_project = String::from("Inbox");

    // Regex for a valid URL (simple, not exhaustive)
    let url_re = Regex::new(r"^https?://[\w.-]+(:\d+)?(/[\w.-]*)*$").unwrap();

    // Prompt for base URL
    loop {
        print!("Enter your Vikunja base URL (e.g. https://vikunja.example.com): ");
        io::stdout().flush().unwrap();
        base_url.clear();
        io::stdin().read_line(&mut base_url).unwrap();
        base_url = base_url.trim().to_string();
        // Remove trailing /api/v1 if present
        if base_url.ends_with("/api/v1") {
            base_url = base_url.trim_end_matches("/api/v1").to_string();
        }
        if !url_re.is_match(&base_url) {
            println!("Invalid URL. Please enter a valid http(s) URL.");
            continue;
        }
        break;
    }

    let mut api_key = String::new();
    // Prompt for API key
    print!("Paste your Vikunja API token (from the web interface): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut api_key).unwrap();
    let api_key = api_key.trim().to_string();

    // Ask about default project
    print!("Default project is 'Inbox'. Change it? (y/N): ");
    io::stdout().flush().unwrap();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer).unwrap();
    if answer.trim().to_lowercase() == "y" {
        print!("Enter new default project name: ");
        io::stdout().flush().unwrap();
        default_project.clear();
        io::stdin().read_line(&mut default_project).unwrap();
        default_project = default_project.trim().to_string();
    }

    // When constructing config, use crate::config::CriaConfig
    let config = CriaConfig {
        api_url: base_url.clone(),
        api_key: Some(api_key),
        api_key_file: None,
        default_project: Some(default_project),
        default_filter: None,
        auto_refresh: None,
        refresh_interval_seconds: None,
        quick_actions: None,
        table_columns: None,
        column_layouts: None,
        active_layout: None,
    };

    // Save config
    let config_path = match std::env::var("XDG_CONFIG_HOME") {
        Ok(val) => PathBuf::from(val).join("cria/config.yaml"),
        Err(_) => {
            let mut home = dirs::home_dir().unwrap();
            home.push(".config/cria/config.yaml");
            home
        }
    };
    if let Some(parent) = config_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    // Offer to backup existing config if present
    if config_path.exists() {
        print!("A config file already exists. Backup and create a new one? (y/N): ");
        io::stdout().flush().unwrap();
        let mut backup_answer = String::new();
        io::stdin().read_line(&mut backup_answer).unwrap();
        if backup_answer.trim().to_lowercase() == "y" {
            let backup_path = config_path.with_extension("yaml.bak");
            if let Err(e) = fs::copy(&config_path, &backup_path) {
                println!("Failed to backup config: {}", e);
            } else {
                println!("Backed up existing config to {}", backup_path.display());
            }
        } else {
            println!("Aborting wizard. No changes made.");
            return None;
        }
    }
    let yaml = serde_yaml::to_string(&config).unwrap();
    fs::write(&config_path, yaml).unwrap();
    println!("Config saved to {}", config_path.display());
    Some(config)
}
