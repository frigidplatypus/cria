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

use crate::tui::app::App;
use crate::tui::events::{Event, EventHandler};
use crate::tui::ui::draw;
use crate::vikunja_client::VikunjaClient as ApiClient;
use crate::debug::debug_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Debug environment variables
    debug_log("Starting CRIA application");
    debug_log(&format!("Environment variables:"));
    debug_log(&format!("  VIKUNJA_URL: {:?}", std::env::var("VIKUNJA_URL")));
    debug_log(&format!("  VIKUNJA_TOKEN: {:?}", std::env::var("VIKUNJA_TOKEN").map(|t| format!("{}...", &t[..t.len().min(8)]))));
    debug_log(&format!("  VIKUNJA_DEFAULT_PROJECT: {:?}", std::env::var("VIKUNJA_DEFAULT_PROJECT")));

    let api_client = Arc::new(Mutex::new(
        ApiClient::new(
            std::env::var("VIKUNJA_API_URL").unwrap_or_else(|_| "http://localhost:3456/api/v1".to_string()),
            std::env::var("VIKUNJA_API_TOKEN").unwrap_or_else(|_| "demo-token".to_string())
        )
    ));
    
    let app = Arc::new(Mutex::new(App::new()));
    
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
    if let Some(first) = tasks.get(0) {
        debug_log(&format!("First task: {:?}", first));
    } else {
        debug_log("No tasks returned from API");
    }
    // Populate saved filters (views) from negative project IDs
    let filters: Vec<(i64, String)> = project_map.iter()
        .filter(|(id, _)| **id < 0)
        .map(|(id, name)| (*id, name.clone()))
        .collect();
    debug_log(&format!("Populated {} saved filters from negative project IDs", filters.len()));
    {
        let mut app_guard = app.lock().await;
        app_guard.update_all_tasks(tasks);
        app_guard.project_map = project_map;
        app_guard.project_colors = project_colors;
        app_guard.set_filters(filters);
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

        if let Event::Key(key) = event_handler.next()? {
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
                    tui::pickers::handle_filter_picker(&mut app_guard, &key);
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
                    },
                    KeyCode::Char('k') => {
                        app_guard.previous_task();
                    },
                    KeyCode::Char('f') => {
                        app_guard.show_filter_picker();
                    },
                    KeyCode::Char('a') => {
                        app_guard.show_quick_add_modal = true;
                    },
                    KeyCode::Char('e') => {
                        app_guard.show_edit_modal = true;
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
                        let filters: Vec<(i64, String)> = project_map.iter()
                            .filter(|(id, _)| **id < 0)
                            .map(|(id, name)| (*id, name.clone()))
                            .collect();
                        {
                            let mut app_guard = app.lock().await;
                            app_guard.update_all_tasks(tasks);
                            app_guard.project_map = project_map;
                            app_guard.project_colors = project_colors;
                            app_guard.set_filters(filters);
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
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
