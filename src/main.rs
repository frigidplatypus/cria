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
                debug_log("The app will still work for viewing, but Quick Add won't function.");
            }
            Err(e) => {
                debug_log(&format!("WARNING: Failed to connect to Vikunja API: {}", e));
                debug_log("The app will still work for viewing, but Quick Add won't function.");
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
                
                // Handle Escape globally to close any modal
                if key.code == KeyCode::Esc {
                    if app_guard.show_quick_add_modal {
                        app_guard.hide_quick_add_modal();
                        continue;
                    }
                    if app_guard.show_edit_modal {
                        app_guard.hide_edit_modal();
                        continue;
                    }
                    if app_guard.show_confirmation_dialog {
                        app_guard.cancel_confirmation();
                        continue;
                    }
                    if app_guard.show_project_picker {
                        app_guard.hide_project_picker();
                        continue;
                    }
                    if app_guard.show_filter_picker {
                        app_guard.hide_filter_picker();
                        continue;
                    }
                }

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
                                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
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
                } else if app_guard.show_edit_modal {
                    // Handle edit modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_edit_modal();
                        },
                        KeyCode::Enter => {
                            let input = app_guard.get_edit_input().to_string();
                            let task_id = app_guard.editing_task_id;
                            if !input.trim().is_empty() && task_id.is_some() {
                                debug_log(&format!("Updating task ID {} with input: '{}'", task_id.unwrap(), input));
                                app_guard.hide_edit_modal();
                                drop(app_guard);
                                // Update task using API client
                                let api_client_guard = api_client.lock().await;
                                match api_client_guard.update_task_with_magic(task_id.unwrap(), &input).await {
                                    Ok(task) => {
                                        debug_log(&format!("SUCCESS: Task updated successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                                        // Refresh tasks list
                                        drop(api_client_guard);
                                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                                        let mut app_guard = app.lock().await;
                                        app_guard.tasks = tasks;
                                        app_guard.project_map = project_map;
                                        app_guard.project_colors = project_colors;
                                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app_guard.tasks.len()));
                                    }
                                    Err(e) => {
                                        debug_log(&format!("ERROR: Failed to update task: {}", e));
                                    }
                                }
                            } else {
                                debug_log("Empty input or no task selected, not updating task");
                            }
                        },
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_edit();
                        },
                        KeyCode::Left => {
                            app_guard.move_edit_cursor_left();
                        },
                        KeyCode::Right => {
                            app_guard.move_edit_cursor_right();
                        }
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_edit(c);
                        },
                        _ => {},
                    }
                } else if app_guard.show_confirmation_dialog {
                    // Handle confirmation dialog
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            // Confirm the action
                            if let Some(task_id) = app_guard.confirm_action() {
                                debug_log(&format!("Confirmed action for task ID: {}", task_id));
                                
                                // If it was a delete action, we need to call the API
                                if app_guard.pending_action.is_none() { // Action was executed
                                    drop(app_guard);
                                    
                                    // Call delete API
                                    let api_client_guard = api_client.lock().await;
                                    if let Err(e) = api_client_guard.delete_task(task_id).await {
                                        debug_log(&format!("ERROR: Failed to delete task from API: {}", e));
                                    } else {
                                        debug_log(&format!("Task {} deleted from API", task_id));
                                    }
                                    
                                    // Refresh tasks list
                                    drop(api_client_guard);
                                    let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                                    let mut app_guard = app.lock().await;
                                    app_guard.tasks = tasks;
                                    app_guard.project_map = project_map;
                                    app_guard.project_colors = project_colors;
                                }
                            }
                        },
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // Cancel the action
                            app_guard.cancel_confirmation();
                        },
                        _ => {},
                    }
                } else if app_guard.show_project_picker {
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_project_picker();
                        },
                        KeyCode::Enter => {
                            app_guard.select_project_picker();
                        },
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_project_picker();
                        },
                        KeyCode::Up => {
                            app_guard.move_project_picker_up();
                        },
                        KeyCode::Down => {
                            app_guard.move_project_picker_down();
                        },
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_project_picker(c);
                        },
                        _ => {},
                    }
                } else if app_guard.show_filter_picker {
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_filter_picker();
                        },
                        KeyCode::Enter => {
                            app_guard.select_filter_picker();
                        },
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_filter_picker();
                        },
                        KeyCode::Up => {
                            app_guard.move_filter_picker_up();
                        },
                        KeyCode::Down => {
                            app_guard.move_filter_picker_down();
                        },
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_filter_picker(c);
                        },
                        _ => {},
                    }
                } else {
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
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
