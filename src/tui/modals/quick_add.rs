// Quick Add Modal event handler split from modals.rs
use crate::tui::app::state::App;
use crate::tui::app::suggestion_mode::SuggestionMode;
use crossterm::event::{KeyEvent, KeyModifiers};
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;

pub async fn handle_quick_add_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    
    // Handle Ctrl+Z (undo) and Ctrl+Y (redo) in quick add modal
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('z') => {
                debug_log("Quick Add Modal: Undo requested (Ctrl+Z)");
                if let Some(_) = app.undo_last_action() {
                    debug_log("Quick Add Modal: Undo successful");
                } else {
                    debug_log("Quick Add Modal: No action to undo");
                }
                return;
            },
            KeyCode::Char('y') => {
                debug_log("Quick Add Modal: Redo requested (Ctrl+Y)");
                if let Some(_) = app.redo_last_action() {
                    debug_log("Quick Add Modal: Redo successful");
                } else {
                    debug_log("Quick Add Modal: No action to redo");
                }
                return;
            },
            _ => {}
        }
    }
    
    match key.code {
        KeyCode::Esc => {
            app.hide_quick_add_modal();
        },
        KeyCode::Enter => {
            let original_input = app.get_quick_add_input().to_string();
            let mut input = original_input.clone();
            let mut updated = false;
            
            // Process new-label commands
            while let Some(start_idx) = input.find("new-label:") {
                let command_start = start_idx;
                let name_start = start_idx + "new-label:".len();
                
                // Find the end of the command - either next space, or end of input
                let command_end = if input[name_start..].starts_with('[') {
                    // Look for closing bracket
                    input[name_start..].find(']').map(|i| name_start + i + 1)
                        .and_then(|bracket_end| {
                            // Find space after bracket or end of string
                            input[bracket_end..].find(' ').map(|i| bracket_end + i).or(Some(input.len()))
                        })
                        .unwrap_or(input.len())
                } else {
                    // Look for next space or end of string
                    input[name_start..].find(' ').map(|i| name_start + i).unwrap_or(input.len())
                };
                
                let command = &input[command_start..command_end];
                let label_name = command.trim_start_matches("new-label:").trim_matches(['[', ']', ' '].as_ref());
                
                if !label_name.is_empty() {
                    debug_log(&format!("QUICK_ADD: Processing label '{}'", label_name));
                    
                    // Check if label already exists
                    let existing_label = app.label_map.values()
                        .find(|label| label.to_lowercase() == label_name.to_lowercase());
                    
                    if let Some(existing_label_name) = existing_label {
                        debug_log(&format!("Label '{}' already exists", label_name));
                        app.show_toast(format!("Label '{}' already exists", existing_label_name));
                        updated = true;
                        
                        // Replace the command with label syntax to apply it to the task
                        let label_syntax = if label_name.contains(' ') {
                            format!("*[{}]", label_name)
                        } else {
                            format!("*{}", label_name)
                        };
                        input = format!("{}{}{}", &input[..command_start], label_syntax, &input[command_end..]).trim().to_string();
                    } else {
                        debug_log(&format!("QUICK_ADD: Creating new label '{}'", label_name));
                        let api_client_guard = api_client.lock().await;
                        match api_client_guard.create_label(label_name).await {
                            Ok(label) => {
                                debug_log(&format!("SUCCESS: Label created! ID: {:?}, Title: '{}'", label.id, label.title));
                                app.show_toast(format!("Created label '{}'", label.title));
                                if let Some(id) = label.id {
                                    app.label_map.insert(id as i64, label.title.clone());
                                }
                                updated = true;
                                
                                // Replace the command with label syntax to apply it to the task
                                let label_syntax = if label_name.contains(' ') {
                                    format!("*[{}]", label_name)
                                } else {
                                    format!("*{}", label_name)
                                };
                                input = format!("{}{}{}", &input[..command_start], label_syntax, &input[command_end..]).trim().to_string();
                            }
                            Err(e) => {
                                debug_log(&format!("ERROR: Failed to create label: {}", e));
                                app.show_toast(format!("Failed to create label '{}': {}", label_name, e));
                                // Remove the command from input even if creation failed
                                input = format!("{}{}", &input[..command_start], &input[command_end..]).trim().to_string();
                            }
                        }
                    }
                } else {
                    // Remove the command from input if label name is empty
                    input = format!("{}{}", &input[..command_start], &input[command_end..]).trim().to_string();
                }
            }
            
            // Process new-project commands
            while let Some(start_idx) = input.find("new-project:") {
                let command_start = start_idx;
                let name_start = start_idx + "new-project:".len();
                
                // Find the end of the command - either next space, or end of input
                let command_end = if input[name_start..].starts_with('[') {
                    // Look for closing bracket
                    input[name_start..].find(']').map(|i| name_start + i + 1)
                        .and_then(|bracket_end| {
                            // Find space after bracket or end of string
                            input[bracket_end..].find(' ').map(|i| bracket_end + i).or(Some(input.len()))
                        })
                        .unwrap_or(input.len())
                } else {
                    // Look for next space or end of string
                    input[name_start..].find(' ').map(|i| name_start + i).unwrap_or(input.len())
                };
                
                let command = &input[command_start..command_end];
                let project_name = command.trim_start_matches("new-project:").trim_matches(['[', ']', ' '].as_ref());
                
                if !project_name.is_empty() {
                    debug_log(&format!("QUICK_ADD: Processing project '{}'", project_name));
                    
                    // Check if project already exists
                    let existing_project = app.project_map.values()
                        .find(|project| project.to_lowercase() == project_name.to_lowercase());
                    
                    if let Some(existing_project_name) = existing_project {
                        debug_log(&format!("Project '{}' already exists", project_name));
                        app.show_toast(format!("Project '{}' already exists", existing_project_name));
                        updated = true;
                        
                        // Replace the command with project syntax to apply it to the task
                        let project_syntax = if project_name.contains(' ') {
                            format!("+[{}]", project_name)
                        } else {
                            format!("+{}", project_name)
                        };
                        input = format!("{}{}{}", &input[..command_start], project_syntax, &input[command_end..]).trim().to_string();
                    } else {
                        debug_log(&format!("QUICK_ADD: Creating new project '{}'", project_name));
                        let api_client_guard = api_client.lock().await;
                        match api_client_guard.create_project(project_name, "#2196f3").await {
                            Ok(project) => {
                                debug_log(&format!("SUCCESS: Project created! ID: {:?}, Title: '{}'", project.id, project.title));
                                app.show_toast(format!("Created project '{}'", project.title));
                                app.project_map.insert(project.id, project.title.clone());
                                updated = true;
                                
                                // Replace the command with project syntax to apply it to the task
                                let project_syntax = if project_name.contains(' ') {
                                    format!("+[{}]", project_name)
                                } else {
                                    format!("+{}", project_name)
                                };
                                input = format!("{}{}{}", &input[..command_start], project_syntax, &input[command_end..]).trim().to_string();
                            }
                            Err(e) => {
                                debug_log(&format!("ERROR: Failed to create project: {}", e));
                                app.show_toast(format!("Failed to create project '{}': {}", project_name, e));
                                // Remove the command from input even if creation failed
                                input = format!("{}{}", &input[..command_start], &input[command_end..]).trim().to_string();
                            }
                        }
                    }
                } else {
                    // Remove the command from input if project name is empty
                    input = format!("{}{}", &input[..command_start], &input[command_end..]).trim().to_string();
                }
            }
            
            if updated {
                app.quick_add_input = input.clone();
                // Reset cursor position to end of input to avoid out-of-bounds issues
                app.quick_add_cursor_position = input.len();
                app.update_suggestions(&input, input.len());
            }
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
                debug_log(&format!("Auto-completing suggestion: {}", app.suggestions[app.selected_suggestion]));
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
            // Submit the task
            debug_log(&format!("Submitting quick add task with input: '{}'", app.get_quick_add_input()));
            let input = app.get_quick_add_input().to_string();
            if !input.trim().is_empty() {
                debug_log(&format!("QUICK_ADD: Creating task with input: '{}'", input));
                debug_log(&format!("QUICK_ADD: Input length: {}, trimmed length: {}", input.len(), input.trim().len()));
                app.hide_quick_add_modal();
                let default_project_name = app.get_active_default_project();
                debug_log(&format!("QUICK_ADD: Active default project: '{}'", default_project_name));
                debug_log(&format!("QUICK_ADD: Project override active: {:?}", app.active_project_override));
                debug_log(&format!("QUICK_ADD: Current filter ID: {:?}", app.current_filter_id));
                let api_client_guard = api_client.lock().await;
                let default_project_id: Option<u64>;
                // Try to resolve project name to ID
                match app.project_map.iter().find_map(|(id, name)| {
                    if name.trim().eq_ignore_ascii_case(&default_project_name) { Some(*id) } else { None }
                }) {
                    Some(id) => {
                        default_project_id = Some(id as u64);
                        debug_log(&format!("QUICK_ADD: Resolved project '{}' to ID {} via project_map", default_project_name, id));
                    },
                    None => {
                        debug_log(&format!("QUICK_ADD: Project '{}' not found in project_map, trying API lookup...", default_project_name));
                        match api_client_guard.find_or_get_project_id(&default_project_name).await {
                            Ok(Some(api_id)) => {
                                default_project_id = Some(api_id as u64);
                                debug_log(&format!("QUICK_ADD: Resolved project '{}' to ID {} via API", default_project_name, api_id));
                            },
                            Ok(None) => {
                                debug_log(&format!("QUICK_ADD ERROR: Project '{}' not found via API, falling back to project ID 1", default_project_name));
                                default_project_id = Some(1);
                            },
                            Err(e) => {
                                debug_log(&format!("QUICK_ADD ERROR: Exception while looking up project '{}': {}. Falling back to project ID 1", default_project_name, e));
                                default_project_id = Some(1);
                            }
                        }
                    }
                }
                let default_project_id = default_project_id.unwrap_or(1);
                debug_log(&format!("QUICK_ADD: Using default project ID: {} (name: '{}')", default_project_id, default_project_name));
                debug_log("QUICK_ADD: Calling create_task_with_magic...");
                match api_client_guard.create_task_with_magic(&input, default_project_id).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(Local::now());
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
                                app.flash_start = Some(Local::now());
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
