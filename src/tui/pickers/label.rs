// Label Picker event handler
use crate::tui::app::state::App;
use crossterm::event::KeyEvent;

#[allow(dead_code)]
pub fn handle_label_picker(app: &mut App, key: &KeyEvent) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_label_picker();
        },
        KeyCode::Enter => {
            app.select_label_picker();
        },
        KeyCode::Backspace => {
            app.delete_char_from_label_picker();
        },
        KeyCode::Up => {
            app.move_label_picker_up();
        },
        KeyCode::Down => {
            app.move_label_picker_down();
        },
        KeyCode::Char(' ') => {
            app.toggle_label_picker();
        },
        KeyCode::Char(c) => {
            app.add_char_to_label_picker(c);
        },
        _ => {},
    }
}
