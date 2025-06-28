//! Quick Add Modal event handler for the CRIA TUI application.
//!
//! Contains logic for handling key events in the Quick Add modal.
//!
//! Moved from `modals.rs` as part of modularization by modal type.

use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::tui::utils::{debug_log, info_log, warn_log, error_log};
use crate::tui::keybinds::{action_for_keycode, KeyAction};

// Quick Add Modal event handler
// Move logic from modals.rs here

pub enum QuickAddResult {
    Cancel,
    // ...other variants as needed...
}

pub async fn handle_quick_add_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) -> Option<QuickAddResult> {
    use crossterm::event::KeyCode;
    match key.code {
        code if action_for_keycode(&code) == Some(KeyAction::CloseModal) => {
            return Some(QuickAddResult::Cancel);
        }
        code if action_for_keycode(&code) == Some(KeyAction::AddTask) || code == KeyCode::Enter => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.quick_add_cursor_position;
                let input = app.get_quick_add_input();
                // Find the last * or + before the cursor
                if let Some(pos) = input[..cursor].rfind(|c| c == '*' || c == '+') {
                    let prefix_start = pos + 1;
                    let mut new_input = String::new();
                    new_input.push_str(&input[..prefix_start]);
                    new_input.push_str(&suggestion);
                    // Only add a space if at the end of the token or next char is space
                    if input.get(cursor..cursor+1).map_or(true, |c| c == " " || c == "") {
                        new_input.push(' ');
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = prefix_start + suggestion.len() + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = prefix_start + suggestion.len();
                    }
                    app.quick_add_input = new_input;
                }
                let input = app.quick_add_input.clone();
                let cursor = app.quick_add_cursor_position;
                app.update_suggestions(&input, cursor);
                return None;
            }
            let input = app.get_quick_add_input().to_string();
            if !input.trim().is_empty() {
                info_log(app, &format!("Creating task with input: '{}'", input));
                app.hide_quick_add_modal();
                // Get default project ID
                let default_project_id = std::env::var("VIKUNJA_DEFAULT_PROJECT")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse::<u64>()
                    .unwrap_or(1);
                debug_log(app, &format!("Using project ID: {}", default_project_id));
                debug_log(app, "Calling create_task_with_magic...");
                // Create task using API client
                let api_client_guard = api_client.lock().await;
                match api_client_guard.create_task_with_magic(app, &input, default_project_id).await {
                    Ok(task) => {
                        info_log(app, &format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(std::time::Instant::now());
                        app.flash_cycle_count = 0;
                        app.flash_cycle_max = 6;
                        // Refresh tasks list
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        app.apply_task_filter();
                        info_log(app, &format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                        // After filtering, find the new task in the filtered list and select/flash it
                        if let Some(new_id) = task.id.map(|id| id as i64) {
                            if let Some(idx) = app.tasks.iter().position(|t| t.id == new_id) {
                                app.selected_task_index = idx;
                                app.flash_task_id = Some(new_id);
                                app.flash_start = Some(std::time::Instant::now());
                                app.flash_cycle_count = 0;
                                app.flash_cycle_max = 6;
                            }
                        }
                    }
                    Err(e) => {
                        error_log(app, &format!("Failed to create task: {}", e));
                    }
                }
            } else {
                warn_log(app, "Empty input, not creating task");
            }
        },
        code if code == KeyCode::Tab => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.quick_add_cursor_position;
                let input = app.get_quick_add_input();
                if let Some(pos) = input[..cursor].rfind(|c| c == '*' || c == '+') {
                    let prefix_start = pos + 1;
                    let mut new_input = String::new();
                    new_input.push_str(&input[..prefix_start]);
                    new_input.push_str(&suggestion);
                    if input.get(cursor..cursor+1).map_or(true, |c| c == " " || c == "") {
                        new_input.push(' ');
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = prefix_start + suggestion.len() + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = prefix_start + suggestion.len();
                    }
                    app.quick_add_input = new_input;
                }
                let input = app.quick_add_input.clone();
                let cursor = app.quick_add_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        code if action_for_keycode(&code) == Some(KeyAction::MoveDown) => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                app.selected_suggestion = (app.selected_suggestion + 1) % app.suggestions.len();
                let input = app.quick_add_input.clone();
                let cursor = app.quick_add_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        code if action_for_keycode(&code) == Some(KeyAction::MoveUp) => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                if app.selected_suggestion == 0 {
                    app.selected_suggestion = app.suggestions.len() - 1;
                } else {
                    app.selected_suggestion -= 1;
                }
                let input = app.quick_add_input.clone();
                let cursor = app.quick_add_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        code if code == KeyCode::Backspace => {
            app.delete_char_from_quick_add();
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        code if code == KeyCode::Left => {
            app.move_cursor_left();
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        code if code == KeyCode::Right => {
            app.move_cursor_right();
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Char(c) => {
            app.add_char_to_quick_add(c);
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        _ => {}
    }
    None
}

