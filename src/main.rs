use clap::{Arg, Command};

mod tui;
mod vikunja;
mod vikunja_client;
mod vikunja_parser;
mod debug;
mod config;
mod first_run;
mod ui_loop;

use crate::debug::debug_log;
use crate::ui_loop::run_ui;

fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Parse command-line arguments
    let matches = Command::new("cria")
        .about("CRIA - Terminal User Interface for Vikunja task management")
        .version("0.9.2")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .help("Path to config file")
                .value_name("FILE")
        )
        .arg(
            Arg::new("dev-env")
                .long("dev-env")
                .help("Use environment variables instead of config file")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("wizard")
                .long("wizard")
                .help("Run the configuration wizard")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // Debug environment variables
    debug_log("Starting CRIA application");
    debug_log(&format!("Environment variables:"));
    debug_log(&format!("  VIKUNJA_URL: {:?}", std::env::var("VIKUNJA_URL")));
    debug_log(&format!("  VIKUNJA_TOKEN: {:?}", std::env::var("VIKUNJA_TOKEN").map(|t| format!("{}...", &t[..t.len().min(8)]))));
    debug_log(&format!("  VIKUNJA_DEFAULT_PROJECT: {:?}", std::env::var("VIKUNJA_DEFAULT_PROJECT")));

    // Parse flags
    let use_env = matches.get_flag("dev-env");
    let run_wizard = matches.get_flag("wizard");
    let config_path = matches.get_one::<String>("config");

    let (api_url, api_key, default_project, config) = if use_env {
        debug_log("Using environment variables for API config");
        (
            std::env::var("VIKUNJA_API_URL").unwrap_or_else(|_| "http://localhost:3456/api/v1".to_string()),
            std::env::var("VIKUNJA_API_TOKEN").unwrap_or_else(|_| "demo-token".to_string()),
            std::env::var("VIKUNJA_DEFAULT_PROJECT").unwrap_or_else(|_| "Inbox".to_string()),
            None
        )
    } else if run_wizard {
        debug_log("Running config wizard by user request");
        match crate::first_run::first_run_wizard() {
            Some(cfg) => {
                match cfg.api_key {
                    Some(ref api_key) => (cfg.api_url.clone(), api_key.clone(), cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()), Some(cfg)),
                    None => {
                        eprintln!("Error: No API key provided by wizard");
                        std::process::exit(1);
                    }
                }
            },
            None => {
                eprintln!("Wizard failed. Exiting.");
                std::process::exit(1);
            }
        }
    } else {
        match crate::config::CriaConfig::load_from_path(config_path.map(|s| s.as_str())) {
            Some(cfg) => {
                let config_source = if let Some(path) = config_path {
                    format!("custom path: {}", path)
                } else {
                    "default location".to_string()
                };
                debug_log(&format!("Loaded config from {}: api_url={}, api_key=***", config_source, cfg.api_url));
                if cfg.has_api_key_config() {
                    match cfg.get_api_key() {
                        Ok(api_key) => (cfg.api_url.clone(), api_key, cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()), Some(cfg)),
                        Err(e) => {
                            eprintln!("Error loading API key: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    debug_log("Config exists but no API key configured, running first run wizard");
                    match crate::first_run::first_run_wizard() {
                        Some(wizard_cfg) => {
                            match wizard_cfg.api_key {
                                Some(ref api_key) => (wizard_cfg.api_url.clone(), api_key.clone(), wizard_cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()), Some(wizard_cfg)),
                                None => {
                                    eprintln!("Error: No API key provided by wizard");
                                    std::process::exit(1);
                                }
                            }
                        },
                        None => {
                            eprintln!("Setup cancelled");
                            std::process::exit(1);
                        }
                    }
                }
            },
            None => {
                let error_msg = if let Some(path) = config_path {
                    format!("Config file not found at: {}", path)
                } else {
                    "No config found at default location".to_string()
                };
                debug_log(&error_msg);
                
                if config_path.is_some() {
                    // If custom path was specified but file doesn't exist, exit with error
                    eprintln!("Error: {}", error_msg);
                    std::process::exit(1);
                } else {
                    // If default location doesn't exist, run wizard
                    debug_log("Running first run wizard");
                    match crate::first_run::first_run_wizard() {
                        Some(cfg) => {
                            match cfg.api_key {
                                Some(ref api_key) => (cfg.api_url.clone(), api_key.clone(), cfg.default_project.clone().unwrap_or_else(|| "Inbox".to_string()), Some(cfg)),
                                None => {
                                    eprintln!("Error: No API key provided by wizard");
                                    std::process::exit(1);
                                }
                            }
                        },
                        None => {
                            eprintln!("First run wizard failed. Exiting.");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    };

    // Call async main
    if let Err(e) = tokio_main(api_url, api_key, default_project, config) {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn tokio_main(api_url: String, api_key: String, default_project: String, config: Option<crate::config::CriaConfig>) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::tui::app::state::App;
    use crate::vikunja_client::VikunjaClient as ApiClient;
    use crate::debug::debug_log;

    let api_client = Arc::new(Mutex::new(ApiClient::new(api_url, api_key)));
    let app = Arc::new(Mutex::new(App::new_with_config(config.expect("Config required"), default_project.clone())));
    
    // Test API connection
    {
        let api_client_guard = api_client.lock().await;
        match api_client_guard.test_connection().await {
            Ok(true) => {
                debug_log("SUCCESS: Connected to Vikunja API");
            }
            Ok(false) => {
                debug_log("WARNING: Failed to connect to Vikunja API");
                debug_log("The app requires a connection to the api.");
            }
            Err(e) => {
                debug_log(&format!("WARNING: Failed to connect to Vikunja API: {}", e));
                debug_log("The app requires a connection to the api.");
            }
        }
    }

    let client_clone = api_client.clone();

    // Load tasks and projects before starting the UI
    let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
    debug_log(&format!("Fetched {} tasks from API", tasks.len()));
    // Fetch all labels from API
    let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
    debug_log(&format!("Fetched {} labels from API", all_labels.len()));
    if let Some(first) = tasks.get(0) {
        debug_log(&format!("First task: {:?}", first));
    } else {
        debug_log("No tasks returned from API");
    }
    // Fetch saved filters (views) from backend
    let filters = client_clone.lock().await.get_saved_filters().await.unwrap_or_default();
    debug_log(&format!("Fetched {} saved filters from backend", filters.len()));
    {
        let mut app_guard = app.lock().await;
        app_guard.update_all_tasks(tasks);
        app_guard.project_map = project_map;
        app_guard.project_colors = project_colors;
        app_guard.set_filters(filters);
        // Merge all_labels into label_map and label_colors
        for label in all_labels {
            if let Some(id) = label.id {
                app_guard.label_map.insert(id as i64, label.title.clone());
                app_guard.label_colors.insert(id as i64, label.hex_color.unwrap_or_default());
            }
        }
        debug_log(&format!("App all_tasks count: {}", app_guard.all_tasks.len()));
        debug_log(&format!("App tasks count after filter: {}", app_guard.tasks.len()));
        debug_log(&format!("App project_map: {:?}", app_guard.project_map));
        debug_log(&format!("App filters: {:?}", app_guard.filters));
        debug_log(&format!("App filtered_filters: {:?}", app_guard.filtered_filters));
        debug_log(&format!("App filtered_projects: {:?}", app_guard.filtered_projects));
        debug_log(&format!("App show_project_picker: {} show_filter_picker: {}", app_guard.show_project_picker, app_guard.show_filter_picker));
        debug_log(&format!("App keybindings: q(quit), d(toggle), D(delete), f(filter), a(add), e(edit), p(project)"));
        debug_log(&format!("App initial tasks: {:?}", app_guard.tasks));
        debug_log(&format!("App initial filters: {:?}", app_guard.filters));
        debug_log(&format!("App initial project_map: {:?}", app_guard.project_map));
        debug_log(&format!("App filters after set_filters: {:?}", app_guard.filters));
        debug_log(&format!("App filtered_filters after set_filters: {:?}", app_guard.filtered_filters));
        debug_log(&format!("App filter_picker_input: {:?}", app_guard.filter_picker_input));
        debug_log(&format!("App selected_filter_picker_index: {}", app_guard.selected_filter_picker_index));
    }

    // Initialize terminal and enter raw mode is handled in ui_loop

    // Delegate UI loop to ui_loop module
    run_ui(app.clone(), client_clone.clone()).await?;

    // Event loop delegated to ui_loop; inline loop removed

    Ok(())
}

