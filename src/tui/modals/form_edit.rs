use crate::tui::app::state::App;
use crate::tui::app::form_edit_state::FormEditState;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;
use crate::tui::app::picker_context::PickerContext;

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
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::BackTab => {
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Up => {
                if form.field_index == 0 {
                    form.field_index = FormEditState::get_field_count() - 1;
                } else {
                    form.field_index -= 1;
                }
                update_cursor_position(form);
            }
            KeyCode::Down => {
                form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
                update_cursor_position(form);
            }
            KeyCode::Esc => {
                app.hide_form_edit_modal();
            }
            KeyCode::Enter => {
                // Save the task with updated values
                if let Err(e) = save_form_task(app, api_client, client_clone).await {
                    debug_log(&format!("Failed to save task from form: {}", e));
                } else {
                    app.hide_form_edit_modal();
                }
            }
            KeyCode::Char(' ') => {
                // Handle space key for special field types
                match form.field_index {
                    5 => {
                        // Project picker - show project picker modal
                        app.picker_context = PickerContext::FormEditProject;
                        app.show_project_picker();
                    }
                    6 => {
                        // Label picker - show label picker modal
                        app.selected_label_ids = form.label_ids.clone();
                        app.picker_context = PickerContext::FormEditLabel;
                        app.show_label_picker();
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
            &form.assignee_ids,
            form.is_favorite,
            if form.comment.is_empty() { None } else { Some(&form.comment) },
        ).await;
        
        drop(api_client_guard);
        
        match result {
            Ok(task) => {
                debug_log(&format!("SUCCESS: Task updated from form! ID: {:?}, Title: '{}'", task.id, task.title));
                
                // Refresh tasks
                let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
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
