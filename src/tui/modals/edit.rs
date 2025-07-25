use crate::tui::app::state::App;
use crossterm::event::{KeyEvent, KeyModifiers};
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;
use crate::tui::app::suggestion_mode::SuggestionMode;

pub async fn handle_edit_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    
    // Handle Ctrl+Z (undo) and Ctrl+Y (redo) in edit modal
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('z') => {
                debug_log("Edit Modal: Undo requested (Ctrl+Z)");
                if let Some(_) = app.undo_last_action() {
                    debug_log("Edit Modal: Undo successful");
                } else {
                    debug_log("Edit Modal: No action to undo");
                }
                return;
            },
            KeyCode::Char('y') => {
                debug_log("Edit Modal: Redo requested (Ctrl+Y)");
                if let Some(_) = app.redo_last_action() {
                    debug_log("Edit Modal: Redo successful");
                } else {
                    debug_log("Edit Modal: No action to redo");
                }
                return;
            },
            _ => {}
        }
    }
    
    match key.code {
        KeyCode::Esc => {
            app.hide_edit_modal();
        },
        KeyCode::Enter => {
            // Check if we should auto-complete or submit the task
            let should_autocomplete = if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                // Only auto-complete if the current text exactly matches a suggestion prefix
                // This prevents auto-completing when the user has typed a complete, valid label
                let prefix = &app.suggestion_prefix;
                
                // If the suggestion prefix is not an exact match to any existing label/project,
                // then we should auto-complete. If it is an exact match, the user might want to submit.
                let is_exact_match = match app.suggestion_mode {
                    Some(SuggestionMode::Label) => {
                        app.label_map.values().any(|label| label.to_lowercase() == prefix.to_lowercase())
                    },
                    Some(SuggestionMode::Project) => {
                        app.project_map.values().any(|project| project.to_lowercase() == prefix.to_lowercase())
                    },
                    _ => false
                };
                
                // Auto-complete if it's not an exact match, or if the first suggestion is different from the prefix
                !is_exact_match && !app.suggestions.is_empty() && app.suggestions[0].to_lowercase() != prefix.to_lowercase()
            } else {
                false
            };
            
            if should_autocomplete {
                debug_log(&format!("Auto-completing suggestion in edit modal: {}", app.suggestions[app.selected_suggestion]));
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.edit_cursor_position;
                let input = app.get_edit_input();
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
                        app.edit_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() }) + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.edit_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() });
                    }
                    app.edit_input = new_input;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
                return;
            }
            // Submit the edit
            debug_log(&format!("Submitting edit task with input: '{}'", app.get_edit_input()));
            let input = app.get_edit_input().to_string();
            let task_id = app.editing_task_id;
            if !input.trim().is_empty() && task_id.is_some() {
                debug_log(&format!("Updating task ID {} with input: '{}'", task_id.unwrap(), input));
                app.hide_edit_modal();
                let api_client_guard = api_client.lock().await;
                match api_client_guard.update_task_with_magic(task_id.unwrap(), &input).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task updated successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(Local::now());
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
                        app.edit_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() }) + 1;
                    } else {
                        new_input.push_str(&input[cursor..]);
                        app.edit_cursor_position = pos + 1 + 
                            (if suggestion.contains(' ') { suggestion.len() + 2 } else { suggestion.len() });
                    }
                    app.edit_input = new_input;
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
