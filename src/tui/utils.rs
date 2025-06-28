use ratatui::style::Color;
use crate::tui::app::App;

pub fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color::Rgb(r, g, b))
        },
        _ => None,
    }
}

pub fn get_label_color(label_name: &str, app: &App) -> Color {
    // Find the label ID by name
    let id = app.label_map.iter().find_map(|(id, name)| {
        if name == label_name { Some(id) } else { None }
    });
    if let Some(id) = id {
        if let Some(hex) = app.label_colors.get(id) {
            if let Some(color) = hex_to_color(hex) {
                return color;
            }
        }
    }
    Color::Red
}

pub fn get_project_color(project_name: &str, app: &App) -> Color {
    // Find the project ID by name
    let id = app.project_map.iter().find_map(|(id, name)| {
        if name == project_name { Some(id) } else { None }
    });
    if let Some(id) = id {
        if let Some(hex) = app.project_colors.get(id) {
            if let Some(color) = hex_to_color(hex) {
                return color;
            }
        }
    }
    Color::Magenta
}
