// Quick Add Modal event handler split from modals.rs
use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;

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
                if let Some(pos) = input[..cursor].rfind(|c| c == '*' || c == '+') {
                    let mut new_input = String::new();
                    new_input.push_str(&input[..pos]); // Include everything up to but not including the * or +
                    new_input.push(input.chars().nth(pos).unwrap()); // Add the * or + character
                    
                    // Wrap multi-word suggestions in square brackets for proper parsing
                    if suggestion.contains(' ') {
                        new_input.push_str(&format!("[{}]", suggestion));
                    } else {
                        new_input.push_str(&suggestion);
                    }
                    
                    if input.get(cursor..cursor+1).map_or(true, |c| c == " " || c == "") {
                        new_input.push(' ');
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() }) + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() });
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
                let default_project_name = app.default_project_name.trim();
                let api_client_guard = api_client.lock().await;
                let default_project_id = if let Some(id) = app.project_map.iter().find_map(|(id, name)| {
                    if name.trim().eq_ignore_ascii_case(default_project_name) { Some(*id) } else { None }
                }) {
                    id as u64
                } else {
                    api_client_guard.find_or_get_project_id(default_project_name).await.ok().flatten().unwrap_or(1) as u64
                };
                debug_log(&format!("Using default project ID: {} (name: '{}')", default_project_id, default_project_name));
                debug_log("Calling create_task_with_magic...");
                match api_client_guard.create_task_with_magic(&input, default_project_id).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(std::time::Instant::now());
                        app.flash_cycle_count = 0;
                        app.flash_cycle_max = 6;
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        app.apply_task_filter();
                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
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
                    let mut new_input = String::new();
                    new_input.push_str(&input[..pos]); // Include everything up to but not including the * or +
                    new_input.push(input.chars().nth(pos).unwrap()); // Add the * or + character
                    
                    // Wrap multi-word suggestions in square brackets for proper parsing
                    if suggestion.contains(' ') {
                        new_input.push_str(&format!("[{}]", suggestion));
                    } else {
                        new_input.push_str(&suggestion);
                    }
                    
                    if input.get(cursor..cursor+1).map_or(true, |c| c == " " || c == "") {
                        new_input.push(' ');
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() }) + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.quick_add_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() });
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
