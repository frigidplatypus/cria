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
use crate::vikunja::client::VikunjaClient;
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

    let client = VikunjaClient::new();
    let api_client = Arc::new(Mutex::new(
        ApiClient::new(
            std::env::var("VIKUNJA_URL").unwrap_or_else(|_| "http://localhost:3456".to_string()),
            std::env::var("VIKUNJA_TOKEN").unwrap_or_else(|_| "demo-token".to_string())
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
                debug_log("The app will still work for viewing, but Quick Add won't function.");
            }
            Err(e) => {
                debug_log(&format!("WARNING: Failed to connect to Vikunja API: {}", e));
                debug_log("The app will still work for viewing, but Quick Add won't function.");
            }
        }
    }
    let event_handler = EventHandler::new(250);

    let client_clone = client.clone();

    // Load tasks and projects before starting the UI
    let (tasks, project_map, project_colors) = client_clone.get_tasks_with_projects().await.unwrap_or_default();
    {
        let mut app_guard = app.lock().await;
        app_guard.tasks = tasks;
        app_guard.project_map = project_map;
        app_guard.project_colors = project_colors;
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
                    // Handle modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_quick_add_modal();
                        },
                        KeyCode::Enter => {
                            let input = app_guard.get_quick_add_input().to_string();
                            if !input.trim().is_empty() {
                                debug_log(&format!("Creating task with input: '{}'", input));
                                app_guard.hide_quick_add_modal();
                                
                                // Get default project ID
                                let default_project_id = std::env::var("VIKUNJA_DEFAULT_PROJECT")
                                    .unwrap_or_else(|_| "1".to_string())
                                    .parse::<u64>()
                                    .unwrap_or(1);
                                
                                debug_log(&format!("Using project ID: {}", default_project_id));
                                debug_log("Calling create_task_with_magic...");
                                
                                drop(app_guard);
                                
                                // Create task using API client
                                let api_client_guard = api_client.lock().await;
                                match api_client_guard.create_task_with_magic(&input, default_project_id).await {
                                    Ok(task) => {
                                        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                                        
                                        // Refresh tasks list
                                        drop(api_client_guard);
                                        let (tasks, project_map, project_colors) = client_clone.get_tasks_with_projects().await.unwrap_or_default();
                                        let mut app_guard = app.lock().await;
                                        app_guard.tasks = tasks;
                                        app_guard.project_map = project_map;
                                        app_guard.project_colors = project_colors;
                                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app_guard.tasks.len()));
                                    }
                                    Err(e) => {
                                        debug_log(&format!("ERROR: Failed to create task: {}", e));
                                    }
                                }
                            } else {
                                debug_log("Empty input, not creating task");
                            }
                        },
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_quick_add();
                        },
                        KeyCode::Left => {
                            app_guard.move_cursor_left();
                        },
                        KeyCode::Right => {
                            app_guard.move_cursor_right();
                        },
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_quick_add(c);
                        },
                        _ => {},
                    }
                } else {
                    // Handle normal navigation
                    match key.code {
                        KeyCode::Char('q') => {
                            app_guard.quit();
                        },
                        KeyCode::Char('j') => {
                            app_guard.next_task();
                        },
                        KeyCode::Char('k') => {
                            app_guard.previous_task();
                        },
                        KeyCode::Char('i') => {
                            app_guard.toggle_info_pane();
                        },
                        KeyCode::Char('n') | KeyCode::Char('a') => {
                            app_guard.show_quick_add_modal();
                        },
                        KeyCode::Char('d') => {
                            app_guard.toggle_debug_pane();
                        },
                        KeyCode::Char('c') => {
                            if app_guard.show_debug_pane {
                                app_guard.clear_debug_messages();
                            }
                        },
                        _ => {},
                    }
                }
            }
        }

        let app_guard = app.lock().await;
        if !app_guard.running {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
