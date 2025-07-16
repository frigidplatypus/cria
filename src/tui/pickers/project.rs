// Project Picker event handler split from pickers.rs
use crate::tui::app::state::App;
use crossterm::event::KeyEvent;

#[allow(dead_code)]
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
