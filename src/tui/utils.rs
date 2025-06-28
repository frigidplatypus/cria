use chrono::Local;
use ratatui::style::Color;
use crate::tui::app::App;

// Color utilities
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
    // If no color is found, use white (not red)
    Color::White
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
    // If no color is found, use white (not magenta)
    Color::White
}

// String normalization
pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Normalize a string for search and comparison.
/// - Unicode NFKC normalization
/// - Lowercase
/// - Collapse all whitespace to single spaces
/// - Trim leading/trailing whitespace
pub fn normalize_string(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfkc()
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

// Fuzzy matching (simple example)
pub fn simple_fuzzy_match(query: &str, candidate: &str) -> bool {
    let mut q = query.chars();
    let mut c = candidate.chars();
    let mut qch = q.next();
    let mut cch = c.next();
    while let (Some(qc), Some(cc)) = (qch, cch) {
        if qc == cc {
            qch = q.next();
        }
        cch = c.next();
    }
    qch.is_none()
}

/// Returns a score for how well `query` matches `candidate` (higher is better, 0 = no match).
/// Prefers consecutive matches and matches at word boundaries.
pub fn fuzzy_match_score(query: &str, candidate: &str) -> usize {
    let mut score = 0;
    let mut last_match = None;
    let mut candidate_chars = candidate.chars().enumerate();
    for qc in query.chars() {
        let mut found = false;
        while let Some((i, cc)) = candidate_chars.next() {
            if qc.eq_ignore_ascii_case(&cc) {
                // Bonus for consecutive matches
                if let Some(last) = last_match {
                    if i == last + 1 {
                        score += 5;
                    } else {
                        score += 1;
                    }
                } else {
                    score += 1;
                }
                // Bonus for start of word
                if i == 0 || candidate.chars().nth(i - 1).map_or(false, |c| !c.is_alphanumeric()) {
                    score += 3;
                }
                last_match = Some(i);
                found = true;
                break;
            }
        }
        if !found {
            return 0; // Not a match
        }
    }
    score
}

/// Logging helpers with log levels.
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Log a message with a given level. Debug logs only go to the App's debug buffer.
pub fn log(app: &mut App, level: LogLevel, msg: &str) {
    let now = Local::now();
    let formatted = match level {
        LogLevel::Debug => format!("[DEBUG] {}", msg),
        LogLevel::Info => format!("[INFO] {}", msg),
        LogLevel::Warn => format!("[WARN] {}", msg),
        LogLevel::Error => format!("[ERROR] {}", msg),
    };
    app.debug_messages.push((now, formatted));
    // Optionally, truncate buffer to max N messages
    if app.debug_messages.len() > 500 {
        app.debug_messages.drain(0..(app.debug_messages.len() - 500));
    }
}

/// Convenience wrappers
pub fn debug_log(app: &mut App, msg: &str) { log(app, LogLevel::Debug, msg); }
pub fn info_log(app: &mut App, msg: &str) { log(app, LogLevel::Info, msg); }
pub fn warn_log(app: &mut App, msg: &str) { log(app, LogLevel::Warn, msg); }
pub fn error_log(app: &mut App, msg: &str) { log(app, LogLevel::Error, msg); }
