use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use clap::{Arg, Command};

mod tui;
mod vikunja;
mod vikunja_client;
mod vikunja_parser;
mod debug;
mod config;
mod first_run;

use crate::debug::debug_log;

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
    use crate::tui::app::App;
    use crate::tui::events::{Event, EventHandler};
    use crate::tui::ui::main::draw;
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
    let event_handler = EventHandler::new(250);

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

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    loop {
        let app_guard = app.lock().await;
        terminal.draw(|frame| draw(frame, &app_guard))?;
        drop(app_guard);

        match event_handler.next()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    let mut app_guard = app.lock().await;

                    if app_guard.show_help_modal {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app_guard.hide_help_modal();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    if app_guard.show_quick_add_modal {
                        tui::modals::handle_quick_add_modal(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_edit_modal {
                        tui::modals::handle_edit_modal(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_form_edit_modal {
                        tui::modals::handle_form_edit_modal(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_confirmation_dialog {
                        tui::confirmation::handle_confirmation_dialog(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_project_picker {
                        tui::pickers::project::handle_project_picker(&mut app_guard, &key);
                        continue;
                    }
                    if app_guard.show_label_picker {
                        tui::pickers::label::handle_label_picker(&mut app_guard, &key);
                        continue;
                    }
                    if app_guard.show_filter_picker {
                        // Await the async filter picker handler
                        drop(app_guard); // Release lock before await
                        let mut app_guard = app.lock().await;
                        tui::pickers::filter::handle_filter_picker(&mut app_guard, &key, &api_client).await;
                        // Force redraw after filter selection
                        drop(app_guard);
                        let app_guard = app.lock().await;
                        terminal.draw(|frame| draw(frame, &app_guard))?;
                        drop(app_guard);
                        continue;
                    }
                    if app_guard.show_sort_modal {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app_guard.hide_sort_modal();
                            }
                            KeyCode::Up => {
                                if app_guard.selected_sort_index > 0 {
                                    app_guard.selected_sort_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app_guard.selected_sort_index + 1 < app_guard.sort_options.len() {
                                    app_guard.selected_sort_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let sort = match app_guard.selected_sort_index {
                                    0 => crate::tui::app::SortOrder::Default,
                                    1 => crate::tui::app::SortOrder::TitleAZ,
                                    2 => crate::tui::app::SortOrder::TitleZA,
                                    3 => crate::tui::app::SortOrder::PriorityHighToLow,
                                    4 => crate::tui::app::SortOrder::PriorityLowToHigh,
                                    _ => crate::tui::app::SortOrder::Default,
                                };
                                app_guard.apply_sort(sort);
                                app_guard.hide_sort_modal();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    
                    if app_guard.show_quick_actions_modal {
                        match key.code {
                            KeyCode::Esc => {
                            KeyCode::Esc => {
                                app_guard.hide_quick_actions_modal();
                            }
                            KeyCode::Up => {
                                if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                    if !quick_actions.is_empty() && app_guard.selected_quick_action_index > 0 {
                                        app_guard.selected_quick_action_index -= 1;
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                    if !quick_actions.is_empty() && app_guard.selected_quick_action_index + 1 < quick_actions.len() {
                                        app_guard.selected_quick_action_index += 1;
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                    if app_guard.selected_quick_action_index < quick_actions.len() {
                                        let action = quick_actions[app_guard.selected_quick_action_index].clone();
                                        app_guard.hide_quick_actions_modal();
                                        match app_guard.apply_quick_action(&action) {
                                            Ok(_) => {
                                                app_guard.add_debug_message(format!("Quick action applied: {} -> {}", action.key, action.target));
                                                
                                                // Update the task on the server - handle labels differently
                                                let selected_task = app_guard.get_selected_task().cloned();
                                                drop(app_guard);
                                                if let Some(task) = selected_task {
                                                    if action.action == "label" {
                                                        // For label actions, use the specialized label API
                                                        let mut app_guard = app.lock().await;
                                                        if let Some(label_id) = app_guard.label_map.iter().find_map(|(id, name)| {
                                                            if name == &action.target { Some(*id) } else { None }
                                                        }) {
                                                            app_guard.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                                                            drop(app_guard);
                                                            match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                                                                Ok(_) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update successful for task {}", task.id));
                                                                },
                                                                Err(e) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            app_guard.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                                                        }
                                                    } else {
                                                        // For non-label actions, use the general task update
                                                        let api_task = crate::vikunja_client::VikunjaTask {
                                                            id: Some(task.id as u64),
                                                            title: task.title.clone(),
                                                            description: task.description.clone(),
                                                            done: Some(task.done),
                                                            priority: task.priority.map(|p| p as u8),
                                                            due_date: task.due_date,
                                                            project_id: task.project_id as u64,
                                                            labels: None, // Don't update labels via general task update
                                                            assignees: None,
                                                            is_favorite: Some(task.is_favorite),
                                                            start_date: task.start_date,
                                                        };
                                                        match client_clone.lock().await.update_task(&api_task).await {
                                                            Ok(_) => {
                                                                // API update successful
                                                            },
                                                            Err(e) => {
                                                                let mut app_guard = app.lock().await;
                                                                app_guard.add_debug_message(format!("API update failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                let mut app_guard = app.lock().await;
                                                if let Some(task) = app_guard.get_selected_task() {
                                                    app_guard.flash_task_id = Some(task.id);
                                                    app_guard.flash_start = Some(chrono::Local::now());
                                                    app_guard.flash_cycle_count = 0;
                                                    app_guard.flash_cycle_max = 4;
                                                }
                                                
                                                // Update the task on the server - handle labels differently
                                                let selected_task = app_guard.get_selected_task().cloned();
                                                drop(app_guard);
                                                if let Some(task) = selected_task {
                                                    if action.action == "label" {
                                                        // For label actions, use the specialized label API
                                                        let mut app_guard = app.lock().await;
                                                        if let Some(label_id) = app_guard.label_map.iter().find_map(|(id, name)| {
                                                            if name == &action.target { Some(*id) } else { None }
                                                        }) {
                                                            app_guard.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                                                            drop(app_guard);
                                                            match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                                                                Ok(_) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update successful for task {}", task.id));
                                                                },
                                                                Err(e) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            app_guard.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                                                        }
                                                    } else {
                                                        // For non-label actions, use the general task update
                                                        let api_task = crate::vikunja_client::VikunjaTask {
                                                            id: Some(task.id as u64),
                                                            title: task.title.clone(),
                                                            description: task.description.clone(),
                                                            done: Some(task.done),
                                                            priority: task.priority.map(|p| p as u8),
                                                            due_date: task.due_date,
                                                            project_id: task.project_id as u64,
                                                            labels: None, // Don't update labels via general task update
                                                            assignees: None,
                                                            is_favorite: Some(task.is_favorite),
                                                            start_date: task.start_date,
                                                        };
                                                        match client_clone.lock().await.update_task(&api_task).await {
                                                            Ok(_) => {
                                                                // API update successful
                                                            },
                                                            Err(e) => {
                                                                let mut app_guard = app.lock().await;
                                                                app_guard.add_debug_message(format!("API update failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                let mut app_guard = app.lock().await;
                                                if let Some(task) = app_guard.get_selected_task() {
                                                    app_guard.flash_task_id = Some(task.id);
                                                    app_guard.flash_start = Some(chrono::Local::now());
                                                    app_guard.flash_cycle_count = 0;
                                                    app_guard.flash_cycle_max = 4;
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Quick action error: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                    if let Some((idx, action)) = quick_actions.iter().enumerate().find(|(_, a)| a.key == c.to_string()) {
                                        let action = action.clone();
                                        app_guard.hide_quick_actions_modal();
                                        match app_guard.apply_quick_action(&action) {
                                            Ok(_) => {
                                                app_guard.add_debug_message(format!("Quick action applied: {} -> {}", action.key, action.target));
                                                
                                                // Update the task on the server - handle labels differently
                                                let selected_task = app_guard.get_selected_task().cloned();
                                                drop(app_guard);
                                                if let Some(task) = selected_task {
                                                    if action.action == "label" {
                                                        // For label actions, use the specialized label API
                                                        let mut app_guard = app.lock().await;
                                                        if let Some(label_id) = app_guard.label_map.iter().find_map(|(id, name)| {
                                                            if name == &action.target { Some(*id) } else { None }
                                                        }) {
                                                            app_guard.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                                                            drop(app_guard);
                                                            match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                                                                Ok(_) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update successful for task {}", task.id));
                                                                },
                                                                Err(e) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            app_guard.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                                                        }
                                                    } else {
                                                        // For non-label actions, use the general task update
                                                        let api_task = crate::vikunja_client::VikunjaTask {
                                                            id: Some(task.id as u64),
                                                            title: task.title.clone(),
                                                            description: task.description.clone(),
                                                            done: Some(task.done),
                                                            priority: task.priority.map(|p| p as u8),
                                                            due_date: task.due_date,
                                                            project_id: task.project_id as u64,
                                                            labels: None, // Don't update labels via general task update
                                                            assignees: None,
                                                            is_favorite: Some(task.is_favorite),
                                                            start_date: task.start_date,
                                                        };
                                                        match client_clone.lock().await.update_task(&api_task).await {
                                                            Ok(_) => {
                                                                // API update successful
                                                            },
                                                            Err(e) => {
                                                                let mut app_guard = app.lock().await;
                                                                app_guard.add_debug_message(format!("API update failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                let mut app_guard = app.lock().await;
                                                app_guard.selected_quick_action_index = idx;
                                                if let Some(task) = app_guard.get_selected_task() {
                                                    app_guard.flash_task_id = Some(task.id);
                                                    app_guard.flash_start = Some(chrono::Local::now());
                                                    app_guard.flash_cycle_count = 0;
                                                    app_guard.flash_cycle_max = 4;
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Quick action error: {}", e));
                                            }
                                                
                                                // Update the task on the server - handle labels differently
                                                let selected_task = app_guard.get_selected_task().cloned();
                                                drop(app_guard);
                                                if let Some(task) = selected_task {
                                                    if action.action == "label" {
                                                        // For label actions, use the specialized label API
                                                        let mut app_guard = app.lock().await;
                                                        if let Some(label_id) = app_guard.label_map.iter().find_map(|(id, name)| {
                                                            if name == &action.target { Some(*id) } else { None }
                                                        }) {
                                                            app_guard.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                                                            drop(app_guard);
                                                            match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                                                                Ok(_) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update successful for task {}", task.id));
                                                                },
                                                                Err(e) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            app_guard.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                                                        }
                                                    } else {
                                                        // For non-label actions, use the general task update
                                                        let api_task = crate::vikunja_client::VikunjaTask {
                                                            id: Some(task.id as u64),
                                                            title: task.title.clone(),
                                                            description: task.description.clone(),
                                                            done: Some(task.done),
                                                            priority: task.priority.map(|p| p as u8),
                                                            due_date: task.due_date,
                                                            project_id: task.project_id as u64,
                                                            labels: None, // Don't update labels via general task update
                                                            assignees: None,
                                                            is_favorite: Some(task.is_favorite),
                                                            start_date: task.start_date,
                                                        };
                                                        match client_clone.lock().await.update_task(&api_task).await {
                                                            Ok(_) => {
                                                                // API update successful
                                                            },
                                                            Err(e) => {
                                                                let mut app_guard = app.lock().await;
                                                                app_guard.add_debug_message(format!("API update failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                let mut app_guard = app.lock().await;
                                                app_guard.selected_quick_action_index = idx;
                                                if let Some(task) = app_guard.get_selected_task() {
                                                    app_guard.flash_task_id = Some(task.id);
                                                    app_guard.flash_start = Some(chrono::Local::now());
                                                    app_guard.flash_cycle_count = 0;
                                                    app_guard.flash_cycle_max = 4;
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Quick action error: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        continue;
                    }

                    // Handle Ctrl key combinations first
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        match key.code {
                            KeyCode::Char('z') => {
                                // Ctrl+Z - Undo
                                if let Some(task_id) = app_guard.undo_last_action() {
                                    // Update the corresponding task in all_tasks
                                    let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                    if let Some(updated_task) = updated_task {
                                        if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                            *task = updated_task;
                                        }
                                    }
                                    // Show visual feedback
                                    app_guard.add_debug_message("Undo operation completed".to_string());
                                } else {
                                    app_guard.add_debug_message("Nothing to undo".to_string());
                                }
                            },
                            KeyCode::Char('y') => {
                                // Ctrl+Y - Redo
                                if let Some(task_id) = app_guard.redo_last_action() {
                                    // Update the corresponding task in all_tasks
                                    let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                    if let Some(updated_task) = updated_task {
                                        if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                            *task = updated_task;
                                        }
                                    }
                                    // Show visual feedback
                                    app_guard.add_debug_message("Redo operation completed".to_string());
                                } else {
                                    app_guard.add_debug_message("Nothing to redo".to_string());
                                }
                            },
                            _ => {}
                        }
                        continue; // Skip the regular key handling for Ctrl combinations
                    }

                    // Handle quick action mode
                    if app_guard.quick_action_mode {
                        // Check if quick action mode has expired
                        if app_guard.is_quick_action_mode_expired() {
                            app_guard.exit_quick_action_mode();
                        } else {
                            match key.code {
                                KeyCode::Char(' ') => {
                                    // Space to cancel quick action mode
                                    app_guard.exit_quick_action_mode();
                                }
                                KeyCode::Esc => {
                                    // Escape to cancel quick action mode
                                    app_guard.exit_quick_action_mode();
                                }
                                KeyCode::Char(c) => {
                                    // Look for quick action with this key
                                    if let Some(action) = app_guard.config.get_quick_action(&c.to_string()) {
                                        let action = action.clone();
                                        app_guard.exit_quick_action_mode();
                                        match app_guard.apply_quick_action(&action) {
                                            Ok(_) => {
                                                app_guard.add_debug_message(format!("Quick action applied: {} -> {}", action.key, action.target));
                                                // Update the task on the server - handle labels differently
                                                let selected_task = app_guard.get_selected_task().cloned();
                                                drop(app_guard);
                                                if let Some(task) = selected_task {
                                                    if action.action == "label" {
                                                        // For label actions, use the specialized label API
                                                        let mut app_guard = app.lock().await;
                                                        if let Some(label_id) = app_guard.label_map.iter().find_map(|(id, name)| {
                                                            if name == &action.target { Some(*id) } else { None }
                                                        }) {
                                                            app_guard.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                                                            drop(app_guard);
                                                            match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                                                                Ok(_) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update successful for task {}", task.id));
                                                                },
                                                                Err(e) => {
                                                                    let mut app_guard = app.lock().await;
                                                                    app_guard.add_debug_message(format!("Label API update failed: {}", e));
                                                                }
                                                            }
                                                        } else {
                                                            app_guard.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                                                        }
                                                    } else {
                                                        // For non-label actions, use the general task update
                                                        let api_task = crate::vikunja_client::VikunjaTask {
                                                            id: Some(task.id as u64),
                                                            title: task.title.clone(),
                                                            description: task.description.clone(),
                                                            done: Some(task.done),
                                                            priority: task.priority.map(|p| p as u8),
                                                            due_date: task.due_date,
                                                            project_id: task.project_id as u64,
                                                            labels: None, // Don't update labels via general task update
                                                            assignees: None,
                                                            is_favorite: Some(task.is_favorite),
                                                            start_date: task.start_date,
                                                        };
                                                        match client_clone.lock().await.update_task(&api_task).await {
                                                            Ok(_) => {
                                                                // API update successful
                                                            },
                                                            Err(e) => {
                                                                let mut app_guard = app.lock().await;
                                                                app_guard.add_debug_message(format!("API update failed: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                                let mut app_guard = app.lock().await;
                                                if let Some(task) = app_guard.get_selected_task() {
                                                    app_guard.flash_task_id = Some(task.id);
                                                    app_guard.flash_start = Some(chrono::Local::now());
                                                    app_guard.flash_cycle_count = 0;
                                                    app_guard.flash_cycle_max = 4;
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Quick action error: {}", e));
                                            }
                                        }
                                    } else {
                                        app_guard.exit_quick_action_mode();
                                        app_guard.add_debug_message(format!("No quick action configured for key: {}", c));
                                    }
                                }
                                _ => {
                                    // Any other key exits quick action mode
                                    app_guard.exit_quick_action_mode();
                                }
                            }
                            continue;
                        }
                    }

                    // Main app key handling (outside modals)
                    match key.code {
                        KeyCode::Char(' ') => {
                            // Space key shows quick actions modal
                            app_guard.show_quick_actions_modal();
                        },
                        KeyCode::Char('q') => {
                            app_guard.quit();
                            break;
                        },
                        KeyCode::Char('Q') => {
                            app_guard.quit();
                            break;
                        }
                        KeyCode::Char('d') => {
                            // Toggle task completion
                            if let Some(task_id) = app_guard.toggle_task_completion() {
                                // Get the new done state before borrowing mutably
                                let new_done_state = app_guard.tasks.iter().find(|t| t.id == task_id).unwrap().done;
                                // Find the task in all_tasks and update it too
                                if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                    task.done = new_done_state;
                                }
                                // Update on server
                                let selected_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                drop(app_guard);
                                if let Some(task) = selected_task {
                                    let api_task = crate::vikunja_client::VikunjaTask {
                                        id: Some(task.id as u64),
                                        title: task.title.clone(),
                                        description: None, // Not editing description here
                                        done: Some(task.done),
                                        priority: task.priority.map(|p| p as u8),
                                        due_date: task.due_date,
                                        project_id: task.project_id as u64,
                                        labels: None, // Not editing labels here
                                        assignees: None, // Not editing assignees here
                                        is_favorite: Some(task.is_favorite),
                                        start_date: task.start_date,
                                    };
                                    match client_clone.lock().await.update_task(&api_task).await {
                                        Ok(_) => {
                                            // Task completion update successful
                                        },
                                        Err(e) => {
                                            eprintln!("Failed to update task completion: {}", e);
                                        }
                                    }
                                    match client_clone.lock().await.update_task(&api_task).await {
                                        Ok(_) => {
                                            // Task completion update successful
                                        },
                                        Err(e) => {
                                            eprintln!("Failed to update task completion: {}", e);
                                        }
                                    }
                                }
                            }
                        },
                        KeyCode::Char('D') => {
                            app_guard.request_delete_task();
                        }
                        KeyCode::Char('j') => {
                            app_guard.next_task();
                        }
                        KeyCode::Char('k') => {
                            app_guard.previous_task();
                        }
                        KeyCode::Char('g') => {
                            app_guard.jump_to_top();
                        }
                        KeyCode::Char('G') => {
                            app_guard.jump_to_bottom();
                        }
                        KeyCode::Char('f') => {
                            app_guard.show_filter_picker();
                        }
                        KeyCode::Char('a') => {
                            app_guard.show_quick_add_modal = true;
                        }
                        KeyCode::Char('e') => {
                            app_guard.show_edit_modal();
                        }
                        KeyCode::Char('E') => {
                            app_guard.show_form_edit_modal();
                        }
                        KeyCode::Char('r') => {
                            // Refresh tasks, projects, and labels from API
                            drop(app_guard); // Release lock before await
                            let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                            debug_log(&format!("[r] Refreshed {} tasks from API", tasks.len()));
                            let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
                            debug_log(&format!("[r] Refreshed {} labels from API", all_labels.len()));
                            let filters = client_clone.lock().await.get_saved_filters().await.unwrap_or_default();
                            debug_log(&format!("[r] Refreshed {} saved filters from backend", filters.len()));
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
                            app_guard.add_debug_message("Tasks, projects, and labels refreshed from API (r)".to_string());
                            // Show toast notification
                            app_guard.toast_notification = Some("Refreshed tasks, projects, and labels from API".to_string());
                            app_guard.toast_notification_start = Some(chrono::Local::now());
                        }
                        _ => {}
                    }
                }
            }
            Event::Tick => {
                // On every tick, redraw to allow flash animation and clear expired notifications
                let mut app_guard = app.lock().await;
                // Clear expired layout notification (cleanup old notifications)
                if let Some(start_time) = app_guard.layout_notification_start {
                    if chrono::Local::now().signed_duration_since(start_time).num_seconds() >= 2 {
                        app_guard.layout_notification = None;
                        app_guard.layout_notification_start = None;
                    }
                }
                // Clear expired toast notification
                if let Some(start_time) = app_guard.toast_notification_start {
                    if chrono::Local::now().signed_duration_since(start_time).num_seconds() >= 2 {
                        app_guard.toast_notification = None;
                        app_guard.toast_notification_start = None;
                    }
                }
                terminal.draw(|frame| draw(frame, &app_guard))?;
                drop(app_guard);
            }
        }
        // Exit loop if quit was requested (e.g., via confirmation dialog)
        let app_guard = app.lock().await;
        if !app_guard.running {
            break;
        }
        drop(app_guard);
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
