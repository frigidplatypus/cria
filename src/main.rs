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
use crate::tui::keybinds::{action_for_keycode, KeyAction};
use crate::tui::modals::edit::EditResult;
use crate::tui::modals::quick_add::QuickAddResult;

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
                    debug_log(&mut app_guard, &format!("[event_loop] Key event received: {:?}", key));

                    if app_guard.show_quick_add_modal {
                        if let Some(result) = tui::modals::handle_quick_add_modal(&mut app_guard, &key, &api_client, &client_clone).await {
                            match result {
                                QuickAddResult::Cancel => {
                                    app_guard.show_quick_add_modal = false;
                                }
                                // Handle other QuickAddResult variants as needed
                            }
                        }
                        continue;
                    }
                    if app_guard.show_edit_modal {
                        if let Some(result) = tui::modals::handle_edit_modal(&mut app_guard, &key, &api_client, &client_clone).await {
                            match result {
                                EditResult::Cancel => {
                                    app_guard.show_edit_modal = false;
                                }
                                // Handle other EditResult variants as needed
                            }
                        }
                        continue;
                    }
                    if app_guard.show_confirmation_dialog {
                        tui::confirmation::handle_confirmation_dialog(&mut app_guard, &key, &api_client, &client_clone).await;
                        app_guard.show_confirmation_dialog = false;
                        continue;
                    }
                    // Always handle project picker first if open
                    if app_guard.show_project_picker {
                        debug_log(&mut app_guard, &format!("[event_loop] show_project_picker=true, key: {:?}", key));
                        app_guard.update_filtered_projects();
                        let picker_result = tui::pickers::handle_project_picker(&mut app_guard, &key);
                        if let crate::tui::pickers::project::PickerResult::Cancel = picker_result {
                            app_guard.show_project_picker = false;
                        }
                        continue;
                    }
                    // Always handle filter picker next if open
                    if app_guard.show_filter_picker {
                        drop(app_guard); // Release lock before await
                        let mut app_guard = app.lock().await;
                        let picker_result = tui::pickers::handle_filter_picker(&mut app_guard, &key, &api_client).await;
                        if let crate::tui::pickers::filter::PickerResult::Cancel = picker_result {
                            app_guard.show_filter_picker = false;
                        }
                        // Force redraw after filter selection
                        drop(app_guard);
                        let app_guard = app.lock().await;
                        terminal.draw(|frame| draw(frame, &app_guard))?;
                        drop(app_guard);
                        continue;
                    }
                    if app_guard.show_keybinds_modal {
                        if let Some(action) = action_for_keycode(&key.code) {
                            if action == KeyAction::CloseModal {
                                app_guard.show_keybinds_modal = false;
                            }
                        }
                        continue;
                    }
                    // Main app key handling (outside modals)
                    if let Some(action) = action_for_keycode(&key.code) {
                        match action {
                            KeyAction::Quit => {
                                app_guard.quit();
                                break;
                            },
                            KeyAction::MoveDown => {
                                app_guard.next_task();
                            },
                            KeyAction::MoveUp => {
                                app_guard.previous_task();
                            },
                            KeyAction::JumpTop => {
                                app_guard.selected_task_index = 0;
                            },
                            KeyAction::JumpBottom => {
                                if !app_guard.tasks.is_empty() {
                                    app_guard.selected_task_index = app_guard.tasks.len() - 1;
                                }
                            },
                            KeyAction::EditTask => {
                                app_guard.show_edit_modal();
                            },
                            KeyAction::AddTask => {
                                app_guard.show_quick_add_modal = true;
                            },
                            KeyAction::ToggleComplete => {
                                app_guard.toggle_task_completion();
                            },
                            KeyAction::DeleteTask => {
                                app_guard.request_delete_task();
                            },
                            KeyAction::CompleteTask => {
                                // Optionally implement if different from toggle
                            },
                            KeyAction::Undo => {
                                app_guard.undo_last_action();
                            },
                            KeyAction::ToggleDebug => {
                                app_guard.show_debug_pane = !app_guard.show_debug_pane;
                            },
                            KeyAction::ShowFilterPicker => {
                                app_guard.show_filter_picker = true;
                            },
                            KeyAction::ShowProjectPicker => {
                                app_guard.show_project_picker = true;
                                app_guard.update_filtered_projects();
                            },
                            KeyAction::AssignProjectToTask => {
                                app_guard.show_project_picker = true;
                                app_guard.project_picker_assign_to_task = true;
                                app_guard.update_filtered_projects();
                            },
                            KeyAction::ToggleInfoPane => {
                                app_guard.show_info_pane = !app_guard.show_info_pane;
                            },
                            KeyAction::ShowKeybinds => {
                                app_guard.show_keybinds_modal = true;
                            },
                            KeyAction::CycleFilterBackward => {
                                app_guard.cycle_task_filter();
                            },
                            KeyAction::CycleFilterForward => {
                                app_guard.cycle_task_filter(); // If you want a different method for forward, change here
                            },
                            KeyAction::ToggleStar => {
                                app_guard.toggle_star_selected_task();
                            },
                            _ => {}
                        }
                    }
                }
            }
            Event::Tick => {
                // No-op for now
            }
        }
    }
    // Clean up terminal state before exit
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
