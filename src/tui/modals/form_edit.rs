use crate::tui::app::state::App;
use crate::tui::app::form_edit_state::FormEditState;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;

pub async fn handle_form_edit_modal(
    app: &mut App, 
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    
    if let Some(form) = app.form_edit_state.as_mut() {
        match key.code {
            KeyCode::Tab => {
                // Save current field before moving
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::BackTab => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Up => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Down => {
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::Esc => {
                app.hide_form_edit_modal();
            }
            KeyCode::Enter => {
                // Save current field before validating and saving
                let current_text = form.get_current_field_text();
                form.set_current_field_text(current_text);
                // Validate the form before saving
                if let Some(form) = app.form_edit_state.as_ref() {
                    let mut errors: Vec<String> = Vec::new();
                    // Title required
                    if form.title.trim().is_empty() {
                        errors.push("Title is required.".to_string());
                    }
                    // Due date format (optional, but if present, must be valid)
                    if let Some(due) = &form.due_date {
                        if !due.trim().is_empty() && chrono::NaiveDate::parse_from_str(due.trim(), "%Y-%m-%d").is_err() {
                            errors.push("Due date must be in YYYY-MM-DD format.".to_string());
                        }
                    }
                    // Start date format (optional, but if present, must be valid)
                    if let Some(start) = &form.start_date {
                        if !start.trim().is_empty() && chrono::NaiveDate::parse_from_str(start.trim(), "%Y-%m-%d").is_err() {
                            errors.push("Start date must be in YYYY-MM-DD format.".to_string());
                        }
                    }
                    // Priority (optional, but if present, must be 0-5)
                    if let Some(priority) = form.priority {
                        if priority < 0 || priority > 5 {
                            errors.push("Priority must be between 0 and 5.".to_string());
                        }
                    }
                    // Project ID (should be valid if set)
                    if form.project_id != 0 && !app.project_map.contains_key(&form.project_id) {
                        errors.push("Selected project does not exist.".to_string());
                    }
                    // Label IDs (should be valid if set)
                    for label_id in &form.label_ids {
                        if !app.label_map.contains_key(label_id) {
                            errors.push(format!("Label ID {} does not exist.", label_id));
                        }
                    }
                    // If errors, show notification and do not submit
                    if !errors.is_empty() {
                        let msg = errors.join("\n");
                        debug_log(&format!("FORM VALIDATION ERROR: {}", msg));
                        app.toast_notification = Some(msg);
                        app.toast_notification_start = Some(Local::now());
                        return;
                    }
                }
                // Save the task with updated values
                if let Err(e) = save_form_task(app, api_client, client_clone).await {
                    debug_log(&format!("Failed to save task from form: {}", e));
                    app.toast_notification = Some(format!("Failed to save: {}", e));
                    app.toast_notification_start = Some(Local::now());
                } else {
                    app.hide_form_edit_modal();
                }
            }
            KeyCode::Char(' ') => {
                // Handle space key for special field types
                match form.field_index {
                    5 => {
                        // Project picker - show project picker modal
                        app.open_project_picker_from_form();
                    }
                    6 => {
                        // Label picker - always wire up: use App method to open label picker from form
                        app.open_label_picker_from_form();
                    }
                    8 => {
                        // Toggle favorite
                        form.is_favorite = !form.is_favorite;
                    }
                    _ => {
                        // For text fields, add space normally
                        add_char_to_current_field(form, ' ');
                    }
                }
            }
            KeyCode::Char(c) => {
                add_char_to_current_field(form, c);
            }
            KeyCode::Backspace => {
                delete_char_from_current_field(form);
            }
            KeyCode::Left => {
                if form.cursor_position > 0 {
                    form.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                let current_text = form.get_current_field_text();
                if form.cursor_position < current_text.len() {
                    form.cursor_position += 1;
                }
            }
            _ => {}
        }
    }
}

fn update_cursor_position(form: &mut FormEditState) {
    let current_text = form.get_current_field_text();
    form.cursor_position = current_text.len();
}

fn add_char_to_current_field(form: &mut FormEditState, c: char) {
    match form.field_index {
        0 => {
            form.title.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        1 => {
            form.description.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        2 => {
            let date_str = form.due_date.get_or_insert_with(String::new);
            date_str.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        3 => {
            let date_str = form.start_date.get_or_insert_with(String::new);
            date_str.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        4 => {
            // Priority field - only accept digits 0-5
            if c.is_ascii_digit() {
                let digit = c.to_digit(10).unwrap() as i32;
                if digit <= 5 {
                    form.priority = Some(digit);
                    form.cursor_position = 1;
                }
            }
        }
        9 => {
            form.comment.insert(form.cursor_position, c);
            form.cursor_position += 1;
        }
        _ => {}
    }
}

fn delete_char_from_current_field(form: &mut FormEditState) {
    match form.field_index {
        0 => {
            if form.cursor_position > 0 && form.cursor_position <= form.title.len() {
                form.cursor_position -= 1;
                form.title.remove(form.cursor_position);
            }
        }
        1 => {
            if form.cursor_position > 0 && form.cursor_position <= form.description.len() {
                form.cursor_position -= 1;
                form.description.remove(form.cursor_position);
            }
        }
        2 => {
            if let Some(ref mut date_str) = form.due_date {
                if form.cursor_position > 0 && form.cursor_position <= date_str.len() {
                    form.cursor_position -= 1;
                    date_str.remove(form.cursor_position);
                    if date_str.is_empty() {
                        form.due_date = None;
                    }
                }
            }
        }
        3 => {
            if let Some(ref mut date_str) = form.start_date {
                if form.cursor_position > 0 && form.cursor_position <= date_str.len() {
                    form.cursor_position -= 1;
                    date_str.remove(form.cursor_position);
                    if date_str.is_empty() {
                        form.start_date = None;
                    }
                }
            }
        }
        4 => {
            form.priority = None;
            form.cursor_position = 0;
        }
        9 => {
            if form.cursor_position > 0 && form.cursor_position <= form.comment.len() {
                form.cursor_position -= 1;
                form.comment.remove(form.cursor_position);
            }
        }
        _ => {}
    }
}

async fn save_form_task(
    app: &mut App,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(form) = &app.form_edit_state {
        debug_log(&format!("Saving task from form: ID {}", form.task_id));
        
        let api_client_guard = api_client.lock().await;
        
        // Update the task using the form data
        let result = api_client_guard.update_task_from_form(
            form.task_id,
            &form.title,
            form.description.as_str(),
            form.due_date.as_deref(),
            form.start_date.as_deref(),
            form.priority,
            form.project_id,
            &form.label_ids,
            &[], // Remove assignees from form mode
            form.is_favorite,
            if form.comment.is_empty() { None } else { Some(&form.comment) },
        ).await;
        
        drop(api_client_guard);
        
        match result {
            Ok(task) => {
                debug_log(&format!("SUCCESS: Task updated from form! ID: {:?}, Title: '{}' Description: {:?}", task.id, task.title, task.description));
                
                // Refresh tasks and inject updated task details
                let (mut tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                for t in &mut tasks {
                    if t.id == task.id {
                        *t = task.clone();
                        break;
                    }
                }
                app.all_tasks = tasks;
                app.project_map = project_map;
                app.project_colors = project_colors;
                app.apply_task_filter();
                
                // Flash the updated task
                app.flash_task_id = Some(task.id);
                app.flash_start = Some(Local::now());
                app.flash_cycle_count = 0;
                app.flash_cycle_max = 6;
                
                debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                Ok(())
            }
            Err(e) => {
                debug_log(&format!("ERROR: Failed to update task from form: {}", e));
                Err(e.into())
            }
        }
    } else {
        Err("No form state available".into())
    }
}
