//! Project Picker event handler for the CRIA TUI application.
//!
//! Contains logic for handling key events in the Project Picker.
//!
//! Moved from `pickers.rs` as part of modularization by picker type.

use crate::tui::app::App;
use crate::tui::keybinds::{action_for_keycode, KeyAction};
use crate::tui::utils::debug_log;
use crossterm::event::KeyEvent;

pub enum PickerResult {
    Cancel,
    Continue,
    // ...other variants as needed...
}

pub fn handle_project_picker(app: &mut App, key: &KeyEvent) -> PickerResult {
    use crossterm::event::KeyCode;
    debug_log(app, &format!("[project_picker] key event: {:?}", key));
    match key.code {
        KeyCode::Esc => {
            debug_log(app, "[project_picker] Esc pressed, closing modal");
            return PickerResult::Cancel;
        }
        KeyCode::Enter => {
            debug_log(app, "[project_picker] Enter pressed");
            if app.project_picker_assign_to_task {
                let selected_id = app.filtered_projects.get(app.selected_project_picker_index).map(|(id, _)| *id);
                if let Some(id) = selected_id {
                    debug_log(app, &format!("[project_picker] Assigning project id {} to selected task", id));
                    if id != -1 {
                        app.assign_project_to_selected_task(id);
                    }
                }
                app.project_picker_assign_to_task = false;
                app.hide_project_picker();
                return PickerResult::Cancel;
            } else {
                debug_log(app, "[project_picker] Selecting project for filter");
                app.select_project_picker();
                return PickerResult::Cancel;
            }
        }
        KeyCode::Backspace => {
            debug_log(app, "[project_picker] Backspace pressed");
            app.delete_char_from_project_picker();
            return PickerResult::Continue;
        }
        KeyCode::Up => {
            debug_log(app, "[project_picker] Up pressed");
            app.move_project_picker_up();
            return PickerResult::Continue;
        }
        KeyCode::Down => {
            debug_log(app, "[project_picker] Down pressed");
            app.move_project_picker_down();
            return PickerResult::Continue;
        }
        KeyCode::Char(c) => {
            debug_log(app, &format!("[project_picker] Char typed: {}", c));
            app.add_char_to_project_picker(c);
            return PickerResult::Continue;
        }
        _ => {
            debug_log(app, &format!("[project_picker] Unhandled key: {:?}", key.code));
            return PickerResult::Continue;
        }
    }
    PickerResult::Cancel // or another default
}

