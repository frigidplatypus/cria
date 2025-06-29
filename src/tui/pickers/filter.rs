//! Filter Picker event handler for the CRIA TUI application.
//!
//! Contains logic for handling key events in the Filter Picker.
//!
//! Moved from `pickers.rs` as part of modularization by picker type.

use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::tui::keybinds::{action_for_keycode, KeyAction};

pub enum PickerResult {
    Cancel,
    Continue,
    // ...other variants as needed...
}

pub async fn handle_filter_picker(app: &mut App, key: &KeyEvent, api_client: &Arc<Mutex<VikunjaClient>>) -> PickerResult {
    use crossterm::event::KeyCode;
    match key.code {
        code if action_for_keycode(&code) == Some(KeyAction::CloseModal) => {
            return PickerResult::Cancel;
        }
        code if action_for_keycode(&code) == Some(KeyAction::ShowFilterPicker) || code == KeyCode::Enter => {
            let (id, name) = app.filtered_filters.get(app.selected_filter_picker_index).cloned().unwrap_or((-1, "No Filter".to_string()));
            app.add_debug_message(format!("Filter picker: Enter pressed, id={}, name={}", id, name));
            if id == -1 {
                app.current_filter_id = None;
                app.add_debug_message("Filter picker: No Filter selected, applying all tasks".to_string());
                app.apply_task_filter();
            } else {
                app.current_filter_id = Some(id);
                app.add_debug_message(format!("Filter picker: Fetching tasks for filter id={}", id));
                match api_client.lock().await.get_tasks_for_filter(app, id).await {
                    Ok(tasks) => {
                        app.add_debug_message(format!("Filter picker: Got {} tasks for filter {}", tasks.len(), id));
                        app.apply_filter_tasks(tasks);
                    },
                    Err(e) => {
                        app.add_debug_message(format!("Filter picker: Failed to fetch tasks for filter {}: {}", id, e));
                    }
                }
            }
            app.hide_filter_picker();
            PickerResult::Cancel
        }
        code if code == KeyCode::Backspace => {
            app.delete_char_from_filter_picker();
            PickerResult::Continue
        }
        code if action_for_keycode(&code) == Some(KeyAction::MoveUp) => {
            app.move_filter_picker_up();
            PickerResult::Continue
        }
        code if action_for_keycode(&code) == Some(KeyAction::MoveDown) => {
            app.move_filter_picker_down();
            PickerResult::Continue
        }
        KeyCode::Char(c) => {
            app.add_char_to_filter_picker(c);
            PickerResult::Continue
        }
        _ => PickerResult::Continue,
    }
}

