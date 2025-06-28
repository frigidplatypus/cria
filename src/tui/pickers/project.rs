//! Project Picker event handler for the CRIA TUI application.
//!
//! Contains logic for handling key events in the Project Picker.
//!
//! Moved from `pickers.rs` as part of modularization by picker type.

use crate::tui::app::App;
use crate::tui::keybinds::{action_for_keycode, KeyAction};
use crossterm::event::KeyEvent;

pub enum PickerResult {
    Cancel,
    // ...other variants as needed...
}

pub fn handle_project_picker(app: &mut App, key: &KeyEvent) -> PickerResult {
    use crossterm::event::KeyCode;
    match key.code {
        code if action_for_keycode(&code) == Some(KeyAction::CloseModal) => {
            return PickerResult::Cancel;
        }
        code if action_for_keycode(&code) == Some(KeyAction::ShowProjectPicker) || code == KeyCode::Enter => {
            app.select_project_picker();
        }
        code if code == KeyCode::Backspace => {
            app.delete_char_from_project_picker();
        }
        code if action_for_keycode(&code) == Some(KeyAction::MoveUp) => {
            app.move_project_picker_up();
        }
        code if action_for_keycode(&code) == Some(KeyAction::MoveDown) => {
            app.move_project_picker_down();
        }
        KeyCode::Char(c) => {
            app.add_char_to_project_picker(c);
        }
        _ => {}
    }
    PickerResult::Cancel // or another default
}

