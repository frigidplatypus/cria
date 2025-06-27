use crate::tui::app::App;
use crossterm::event::KeyEvent;

// Project Picker handler
pub fn handle_project_picker(app: &mut App, key: &KeyEvent) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_project_picker();
        },
        KeyCode::Enter => {
            app.select_project_picker();
        },
        KeyCode::Backspace => {
            app.delete_char_from_project_picker();
        },
        KeyCode::Up => {
            app.move_project_picker_up();
        },
        KeyCode::Down => {
            app.move_project_picker_down();
        },
        KeyCode::Char(c) => {
            app.add_char_to_project_picker(c);
        },
        _ => {},
    }
}

// Filter Picker handler
pub fn handle_filter_picker(app: &mut App, key: &KeyEvent) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_filter_picker();
        },
        KeyCode::Enter => {
            app.select_filter_picker();
        },
        KeyCode::Backspace => {
            app.delete_char_from_filter_picker();
        },
        KeyCode::Up => {
            app.move_filter_picker_up();
        },
        KeyCode::Down => {
            app.move_filter_picker_down();
        },
        KeyCode::Char(c) => {
            app.add_char_to_filter_picker(c);
        },
        _ => {},
    }
}
