use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;

// Quick Add Modal handler
pub async fn handle_quick_add_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_quick_add_modal();
        },
        KeyCode::Enter => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.quick_add_cursor_position;
                let input = app.get_quick_add_input();
                // Find the last * or + before the cursor
                if let Some(pos) = input[..cursor].rfind(|c| c == '*' || c == '+') {
                    let prefix_start = pos + 1;
                    let prefix_end = cursor;
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
                return;
            }
            let input = app.get_quick_add_input().to_string();
            if !input.trim().is_empty() {
                debug_log(&format!("Creating task with input: '{}'", input));
                app.hide_quick_add_modal();
                // Get default project ID
                let default_project_id = std::env::var("VIKUNJA_DEFAULT_PROJECT")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse::<u64>()
                    .unwrap_or(1);
                debug_log(&format!("Using project ID: {}", default_project_id));
                debug_log("Calling create_task_with_magic...");
                // Create task using API client
                let api_client_guard = api_client.lock().await;
                match api_client_guard.create_task_with_magic(&input, default_project_id).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
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
                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
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
                        debug_log(&format!("ERROR: Failed to create task: {}", e));
                    }
                }
            } else {
                debug_log("Empty input, not creating task");
            }
        },
        KeyCode::Tab => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.quick_add_cursor_position;
                let input = app.get_quick_add_input();
                if let Some(pos) = input[..cursor].rfind(|c| c == '*' || c == '+') {
                    let prefix_start = pos + 1;
                    let prefix_end = cursor;
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
        KeyCode::Down => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                app.selected_suggestion = (app.selected_suggestion + 1) % app.suggestions.len();
                let input = app.quick_add_input.clone();
                let cursor = app.quick_add_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Up => {
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
        KeyCode::Backspace => {
            app.delete_char_from_quick_add();
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Left => {
            app.move_cursor_left();
            let input = app.quick_add_input.clone();
            let cursor = app.quick_add_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Right => {
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
        _ => {},
    }
}

// Edit Modal handler
pub async fn handle_edit_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_edit_modal();
        },
        KeyCode::Enter => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.edit_cursor_position;
                let input = app.get_edit_input();
                let mut new_input = input[..cursor].to_string();
                if let Some(pos) = new_input.rfind(|c| c == '*' || c == '+') {
                    new_input.truncate(pos + 1);
                    new_input.push_str(&suggestion);
                    new_input.push(' ');
                    new_input.push_str(&input[cursor..]);
                    app.edit_input = new_input;
                    app.edit_cursor_position = pos + 1 + suggestion.len() + 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
                return;
            }
            let input = app.get_edit_input().to_string();
            let task_id = app.editing_task_id;
            if !input.trim().is_empty() && task_id.is_some() {
                debug_log(&format!("Updating task ID {} with input: '{}'", task_id.unwrap(), input));
                app.hide_edit_modal();
                // Update task using API client
                let api_client_guard = api_client.lock().await;
                match api_client_guard.update_task_with_magic(task_id.unwrap(), &input).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task updated successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(std::time::Instant::now());
                        // Refresh tasks list
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        app.apply_task_filter();
                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                    }
                    Err(e) => {
                        debug_log(&format!("ERROR: Failed to update task: {}", e));
                    }
                }
            } else {
                debug_log("Empty input or no task selected, not updating task");
            }
        },
        KeyCode::Tab => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.edit_cursor_position;
                let input = app.get_edit_input();
                let mut new_input = input[..cursor].to_string();
                if let Some(pos) = new_input.rfind(|c| c == '*' || c == '+') {
                    new_input.truncate(pos + 1);
                    new_input.push_str(&suggestion);
                    new_input.push(' ');
                    new_input.push_str(&input[cursor..]);
                    app.edit_input = new_input;
                    app.edit_cursor_position = pos + 1 + suggestion.len() + 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Down => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                app.selected_suggestion = (app.selected_suggestion + 1) % app.suggestions.len();
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Up => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                if app.selected_suggestion == 0 {
                    app.selected_suggestion = app.suggestions.len() - 1;
                } else {
                    app.selected_suggestion -= 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Backspace => {
            app.delete_char_from_edit();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Left => {
            app.move_edit_cursor_left();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Right => {
            app.move_edit_cursor_right();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Char(c) => {
            app.add_char_to_edit(c);
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        _ => {},
    }
}
