use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::Mutex;

mod tui;
mod vikunja;
mod vikunja_client;
mod vikunja_parser;
mod debug;
mod config;
mod first_run;

use crate::tui::app::App;
use crate::tui::events::{Event, EventHandler};
use crate::tui::ui::main::draw;
use crate::vikunja_client::VikunjaClient as ApiClient;
use crate::debug::debug_log;

fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Debug environment variables
    debug_log("Starting CRIA application");
    debug_log(&format!("Environment variables:"));
    debug_log(&format!("  VIKUNJA_URL: {:?}", std::env::var("VIKUNJA_URL")));
    debug_log(&format!("  VIKUNJA_TOKEN: {:?}", std::env::var("VIKUNJA_TOKEN").map(|t| format!("{}...", &t[..t.len().min(8)]))));
    debug_log(&format!("  VIKUNJA_DEFAULT_PROJECT: {:?}", std::env::var("VIKUNJA_DEFAULT_PROJECT")));

    // Parse --dev-env and --wizard flags
    let use_env = std::env::args().any(|arg| arg == "--dev-env");
    let run_wizard = std::env::args().any(|arg| arg == "--wizard");

    let (api_url, api_key, default_project) = if use_env {
        debug_log("Using environment variables for API config");
        (
            std::env::var("VIKUNJA_API_URL").unwrap_or_else(|_| "http://localhost:3456/api/v1".to_string()),
            std::env::var("VIKUNJA_API_TOKEN").unwrap_or_else(|_| "demo-token".to_string()),
            std::env::var("VIKUNJA_DEFAULT_PROJECT").unwrap_or_else(|_| "Inbox".to_string()),
        )
    } else if run_wizard {
        debug_log("Running config wizard by user request");
        match crate::first_run::first_run_wizard() {
            Some(cfg) => (cfg.api_url, cfg.api_key, cfg.default_project),
            None => {
                eprintln!("Wizard failed. Exiting.");
                std::process::exit(1);
            }
        }
    } else {
        match crate::config::CriaConfig::load() {
            Some(cfg) => {
                debug_log(&format!("Loaded config from YAML: api_url={}, api_key=***", cfg.api_url));
                (cfg.api_url, cfg.api_key, cfg.default_project.unwrap_or_else(|| "Inbox".to_string()))
            },
            None => {
                debug_log("No config found, running first run wizard");
                match crate::first_run::first_run_wizard() {
                    Some(cfg) => (cfg.api_url, cfg.api_key, cfg.default_project),
                    None => {
                        eprintln!("First run wizard failed. Exiting.");
                        std::process::exit(1);
                    }
                }
            }
        }
    };

    // Call async main
    if let Err(e) = tokio_main(api_url, api_key, default_project) {
        eprintln!("Application error: {e}");
        std::process::exit(1);
    }
}

#[tokio::main]
async fn tokio_main(api_url: String, api_key: String, default_project: String) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use crate::tui::app::App;
    use crate::tui::events::{Event, EventHandler};
    use crate::tui::ui::main::draw;
    use crate::vikunja_client::VikunjaClient as ApiClient;
    use crate::debug::debug_log;

    let api_client = Arc::new(Mutex::new(ApiClient::new(api_url, api_key)));
    
    let app = Arc::new(Mutex::new(App::new_with_default_project(default_project.clone())));
    
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

                    if app_guard.show_quick_add_modal {
                        tui::modals::handle_quick_add_modal(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_edit_modal {
                        tui::modals::handle_edit_modal(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_confirmation_dialog {
                        tui::confirmation::handle_confirmation_dialog(&mut app_guard, &key, &api_client, &client_clone).await;
                        continue;
                    }
                    if app_guard.show_project_picker {
                        tui::pickers::handle_project_picker(&mut app_guard, &key);
                        continue;
                    }
                    if app_guard.show_filter_picker {
                        // Await the async filter picker handler
                        drop(app_guard); // Release lock before await
                        let mut app_guard = app.lock().await;
                        tui::pickers::handle_filter_picker(&mut app_guard, &key, &api_client).await;
                        // Force redraw after filter selection
                        drop(app_guard);
                        let app_guard = app.lock().await;
                        terminal.draw(|frame| draw(frame, &app_guard))?;
                        drop(app_guard);
                        continue;
                    }
                    // Main app key handling (outside modals)
                    match key.code {
                        KeyCode::Char('q') => {
                            app_guard.quit();
                            break;
                        },
                        KeyCode::Char('d') => {
                            app_guard.toggle_task_completion();
                        },
                        KeyCode::Char('D') => {
                            app_guard.request_delete_task();
                        },
                        KeyCode::Char('j') => {
                            app_guard.next_task();
                        }
                        KeyCode::Char('k') => {
                            app_guard.previous_task();
                        },
                        KeyCode::Char('g') => {
                            app_guard.jump_to_top();
                        },
                        KeyCode::Char('G') => {
                            app_guard.jump_to_bottom();
                        },
                        KeyCode::Char('f') => {
                            app_guard.show_filter_picker();
                        },
                        KeyCode::Char('a') => {
                            app_guard.show_quick_add_modal = true;
                        },
                        KeyCode::Char('e') => {
                            app_guard.show_edit_modal();
                        },
                        KeyCode::Char('p') => {
                            app_guard.show_project_picker();
                        },
                        KeyCode::Char('r') => {
                            debug_log("Refresh key pressed");
                            app_guard.refreshing = true;
                            drop(app_guard); // Release lock before drawing
                            {
                                let app_guard = app.lock().await;
                                if let Err(e) = terminal.draw(|frame| draw(frame, &app_guard)) {
                                    debug_log(&format!("Error drawing refresh indicator: {}", e));
                            }
                            }
                            // Now do the refresh
                            let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                            let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
                            let filters = client_clone.lock().await.get_saved_filters().await.unwrap_or_default();
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
                                app_guard.refreshing = false;
                            }
                            {
                                let app_guard = app.lock().await;
                                if let Err(e) = terminal.draw(|frame| draw(frame, &app_guard)) {
                                    debug_log(&format!("Error drawing after refresh: {}", e));
                            }
                            }
                            debug_log("Refreshed tasks, projects, and filters from API");
                        },
                        KeyCode::Char('s') => {
                            // Toggle star (favorite) for selected task
                            if let Some(task_id) = app_guard.toggle_star_selected_task() {
                                // Find the task in all_tasks and update it too
                                if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                    task.is_favorite = !task.is_favorite;
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
                                        // Add is_favorite if VikunjaTask supports it
                                    };
                                    let _ = client_clone.lock().await.update_task(&api_task).await;
                                }
                            }
                        },
                        KeyCode::Char('i') => {
                            app_guard.show_info_pane = !app_guard.show_info_pane;
                        },
                        KeyCode::Esc => {
                            // Handle Escape globally to close any modal
                            if app_guard.show_quick_add_modal {
                                app_guard.hide_quick_add_modal();
                            } else if app_guard.show_edit_modal {
                                app_guard.hide_edit_modal();
                            } else if app_guard.show_confirmation_dialog {
                                app_guard.cancel_confirmation();
                            } else if app_guard.show_project_picker {
                                app_guard.hide_project_picker();
                            } else if app_guard.show_filter_picker {
                                app_guard.hide_filter_picker();
                            }
                        },
                        KeyCode::Char('h') => {
                            // Cycle filter backward
                            app_guard.task_filter = match app_guard.task_filter {
                                crate::tui::app::TaskFilter::ActiveOnly => crate::tui::app::TaskFilter::CompletedOnly,
                                crate::tui::app::TaskFilter::All => crate::tui::app::TaskFilter::ActiveOnly,
                                crate::tui::app::TaskFilter::CompletedOnly => crate::tui::app::TaskFilter::All,
                            };
                            app_guard.apply_task_filter();
                            app_guard.selected_task_index = 0;
                            let filter_name = match app_guard.task_filter {
                                crate::tui::app::TaskFilter::ActiveOnly => "Active Tasks Only",
                                crate::tui::app::TaskFilter::All => "All Tasks",
                                crate::tui::app::TaskFilter::CompletedOnly => "Completed Tasks Only",
                            };
                            app_guard.add_debug_message(format!("Switched to filter: {}", filter_name));
                        },
                        KeyCode::Char('l') => {
                            // Cycle filter forward
                            app_guard.cycle_task_filter();
                        },
                        _ => {}
                    }
                }
            }
            Event::Tick => {
                // On every tick, redraw to allow flash animation
                let app_guard = app.lock().await;
                terminal.draw(|frame| draw(frame, &app_guard))?;
                drop(app_guard);
            }
        }

        // After drawing, clear flash if expired (multi-cycle)
        {
            let mut app_guard = app.lock().await;
            if let (Some(_), Some(start)) = (app_guard.flash_task_id, app_guard.flash_start) {
                let elapsed = start.elapsed().as_millis() as u64;
                let cycle = (elapsed / 50) as u8;
                if cycle >= app_guard.flash_cycle_max {
                    app_guard.flash_task_id = None;
                    app_guard.flash_start = None;
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
