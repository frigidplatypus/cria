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

/// Normalize a string for comparison by trimming whitespace and converting to lowercase
pub fn normalize_string(input: &str) -> String {
    input.trim().to_ascii_lowercase()
}

/// Check if a string contains another string (case-insensitive)
pub fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Compare two strings for equality (case-insensitive, trimmed)
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    normalize_string(a) == normalize_string(b)
}

/// Simple fuzzy matching - checks if all characters of needle appear in order in haystack
pub fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    
    let mut needle_chars = needle_lower.chars();
    let mut current_char = needle_chars.next();
    
    for h_char in haystack_lower.chars() {
        if let Some(n_char) = current_char {
            if h_char == n_char {
                current_char = needle_chars.next();
                if current_char.is_none() {
                    return true; // All needle characters found
                }
            }
        }
    }
    
    current_char.is_none() // True if all characters were matched
}

/// Calculate a simple fuzzy match score (0.0 to 1.0, higher is better)
pub fn fuzzy_match_score(haystack: &str, needle: &str) -> f32 {
    if needle.is_empty() {
        return 1.0;
    }
    if haystack.is_empty() {
        return 0.0;
    }
    
    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();
    
    // Exact match gets highest score
    if haystack_lower == needle_lower {
        return 1.0;
    }
    
    // Prefix match gets high score
    if haystack_lower.starts_with(&needle_lower) {
        return 0.9;
    }
    
    // Contains match gets medium score
    if haystack_lower.contains(&needle_lower) {
        return 0.8;
    }
    
    // Fuzzy match gets lower score based on how many characters match
    if fuzzy_match(haystack, needle) {
        let match_ratio = needle.len() as f32 / haystack.len() as f32;
        return 0.5 * match_ratio;
    }
    
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_string() {
        assert_eq!(normalize_string("  Hello World  "), "hello world");
        assert_eq!(normalize_string("Test String"), "test string");
        assert_eq!(normalize_string(""), "");
    }

    #[test]
    fn test_contains_ignore_case() {
        assert!(contains_ignore_case("Hello World", "hello"));
        assert!(contains_ignore_case("Hello World", "WORLD"));
        assert!(contains_ignore_case("Test String", "ring"));
        assert!(!contains_ignore_case("Hello", "world"));
    }

    #[test]
    fn test_equals_ignore_case() {
        assert!(equals_ignore_case("  Hello World  ", "hello world"));
        assert!(equals_ignore_case("TEST", "test"));
        assert!(!equals_ignore_case("hello", "world"));
    }

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("hello world", "hw"));
        assert!(fuzzy_match("Home Maintenance", "hm"));
        assert!(fuzzy_match("test string", "tst"));
        assert!(fuzzy_match("hello", "hello"));
        assert!(!fuzzy_match("hello", "xyz"));
    }

    #[test]
    fn test_fuzzy_match_score() {
        // Exact match should score 1.0
        assert_eq!(fuzzy_match_score("hello", "hello"), 1.0);
        
        // Prefix match should score 0.9
        assert_eq!(fuzzy_match_score("hello world", "hello"), 0.9);
        
        // Contains match should score 0.8
        assert_eq!(fuzzy_match_score("hello world", "world"), 0.8);
        
        // Fuzzy match should score between 0 and 0.5
        let score = fuzzy_match_score("hello world", "hw");
        assert!(score > 0.0 && score <= 0.5);
        
        // No match should score 0.0
        assert_eq!(fuzzy_match_score("hello", "xyz"), 0.0);
    }
}
