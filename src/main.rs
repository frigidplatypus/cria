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
use crate::tui::ui::main::draw;
use crate::vikunja_client::VikunjaClient as ApiClient;
use crate::tui::utils::{debug_log, info_log, warn_log, error_log};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

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
        let mut app_guard = app.lock().await;
        match api_client_guard.test_connection().await {
            Ok(true) => {
                debug_log(&mut app_guard, "SUCCESS: Connected to Vikunja API");
            }
            Ok(false) => {
                debug_log(&mut app_guard, "WARNING: Failed to connect to Vikunja API");
                debug_log(&mut app_guard, "The app requires a connection to the api.");
            }
            Err(e) => {
                debug_log(&mut app_guard, &format!("WARNING: Failed to connect to Vikunja API: {}", e));
                debug_log(&mut app_guard, "The app requires a connection to the api.");
            }
        }
    }
    let event_handler = EventHandler::new(250);

    let client_clone = api_client.clone();

    // Load tasks and projects before starting the UI
    let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
    let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
    // Move filters loading into the app_guard block below
    {
        let mut app_guard = app.lock().await;
        let filters = client_clone.lock().await.get_saved_filters(&mut app_guard).await.unwrap_or_default();
        debug_log(&mut app_guard, "Starting CRIA application");
        debug_log(&mut app_guard, &format!("Environment variables:"));
        debug_log(&mut app_guard, &format!("  VIKUNJA_URL: {:?}", std::env::var("VIKUNJA_URL")));
        debug_log(&mut app_guard, &format!("  VIKUNJA_TOKEN: {:?}", std::env::var("VIKUNJA_TOKEN").map(|t| format!("{}...", &t[..t.len().min(8)]))));
        debug_log(&mut app_guard, &format!("  VIKUNJA_DEFAULT_PROJECT: {:?}", std::env::var("VIKUNJA_DEFAULT_PROJECT")));
        debug_log(&mut app_guard, &format!("Fetched {} tasks from API", tasks.len()));
        debug_log(&mut app_guard, &format!("Fetched {} labels from API", all_labels.len()));
        if let Some(first) = tasks.get(0) {
            debug_log(&mut app_guard, &format!("First task: {:?}", first));
        } else {
            debug_log(&mut app_guard, "No tasks returned from API");
        }
        debug_log(&mut app_guard, &format!("Fetched {} saved filters from backend", filters.len()));
        {
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
            let all_tasks_count = app_guard.all_tasks.len();
            debug_log(&mut app_guard, &format!("App all_tasks count: {}", all_tasks_count));
            let tasks_count = app_guard.tasks.len();
            debug_log(&mut app_guard, &format!("App tasks count after filter: {}", tasks_count));
            let project_map = format!("{:?}", app_guard.project_map);
            debug_log(&mut app_guard, &format!("App project_map: {}", project_map));
            let filters = format!("{:?}", app_guard.filters);
            debug_log(&mut app_guard, &format!("App filters: {}", filters));
            let filtered_filters = format!("{:?}", app_guard.filtered_filters);
            debug_log(&mut app_guard, &format!("App filtered_filters: {}", filtered_filters));
            let filtered_projects = format!("{:?}", app_guard.filtered_projects);
            debug_log(&mut app_guard, &format!("App filtered_projects: {}", filtered_projects));
            let show_project_picker = app_guard.show_project_picker;
            let show_filter_picker = app_guard.show_filter_picker;
            debug_log(&mut app_guard, &format!("App show_project_picker: {} show_filter_picker: {}", show_project_picker, show_filter_picker));
            debug_log(&mut app_guard, &format!("App keybindings: q(quit), d(toggle), D(delete), f(filter), a(add), e(edit), p(project)"));
            let initial_tasks = format!("{:?}", app_guard.tasks);
            debug_log(&mut app_guard, &format!("App initial tasks: {}", initial_tasks));
            let initial_filters = format!("{:?}", app_guard.filters);
            debug_log(&mut app_guard, &format!("App initial filters: {}", initial_filters));
            let initial_project_map = format!("{:?}", app_guard.project_map);
            debug_log(&mut app_guard, &format!("App initial project_map: {}", initial_project_map));
            let filters_after = format!("{:?}", app_guard.filters);
            debug_log(&mut app_guard, &format!("App filters after set_filters: {}", filters_after));
            let filtered_filters_after = format!("{:?}", app_guard.filtered_filters);
            debug_log(&mut app_guard, &format!("App filtered_filters after set_filters: {}", filtered_filters_after));
        }
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
                            let mut app_guard = app.lock().await;
                            debug_log(&mut app_guard, "Refresh key pressed");
                            app_guard.refreshing = true;
                            drop(app_guard); // Release lock before drawing
                            {
                                let app_guard_draw = app.lock().await;
                                if let Err(e) = terminal.draw(|frame| draw(frame, &app_guard_draw)) {
                                    let mut app_guard_log = app.lock().await;
                                    debug_log(&mut app_guard_log, &format!("Error drawing refresh indicator: {}", e));
                                }
                            }
                            // Now do the refresh
                            let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                            let all_labels = client_clone.lock().await.get_all_labels().await.unwrap_or_default();
                            let mut app_guard = app.lock().await;
                            let filters = client_clone.lock().await.get_saved_filters(&mut app_guard).await.unwrap_or_default();
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
                            drop(app_guard);
                            {
                                let app_guard_draw = app.lock().await;
                                if let Err(e) = terminal.draw(|frame| draw(frame, &app_guard_draw)) {
                                    let mut app_guard_log = app.lock().await;
                                    debug_log(&mut app_guard_log, &format!("Error drawing after refresh: {}", e));
                                }
                            }
                            let mut app_guard = app.lock().await;
                            debug_log(&mut app_guard, "Refreshed tasks, projects, and filters from API");
                        },
                        KeyCode::Char('s') => {
                            let mut app_guard = app.lock().await;
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
                                    let mut app_guard = app.lock().await;
                                    let _ = client_clone.lock().await.update_task(&mut app_guard, &api_task).await;
                                }
                            }
                        },
                        KeyCode::Char('i') => {
                            app_guard.show_info_pane = !app_guard.show_info_pane;
                        },
                        KeyCode::Char('I') => {
                            app_guard.show_debug_pane = !app_guard.show_debug_pane;
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
                        KeyCode::Char('u') => {
                            app_guard.undo_last_action();
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
