use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy)]
pub enum ThemeMode {
    Dark,   // Default assumption
    Light,
    Auto,   // Try to detect, fallback to Dark
}

pub struct ColorHelper {
    theme_mode: ThemeMode,
}

impl ColorHelper {
    pub fn new(theme_mode: ThemeMode) -> Self {
        Self { theme_mode }
    }
    
    pub fn is_color_problematic(&self, hex_color: &str) -> bool {
        match self.theme_mode {
            ThemeMode::Dark => self.is_color_too_dark(hex_color),
            ThemeMode::Light => self.is_color_too_light(hex_color),
            ThemeMode::Auto => {
                // Try to detect, fallback to dark theme behavior
                if let Some(is_dark_theme) = self.detect_terminal_theme() {
                    if is_dark_theme {
                        self.is_color_too_dark(hex_color)
                    } else {
                        self.is_color_too_light(hex_color)
                    }
                } else {
                    self.is_color_too_dark(hex_color) // Default to dark theme
                }
            }
        }
    }
    
    fn is_color_too_dark(&self, hex_color: &str) -> bool {
        let brightness = self.calculate_brightness(hex_color);
        brightness < 0.4 // Too dark for dark theme
    }
    
    fn is_color_too_light(&self, hex_color: &str) -> bool {
        let brightness = self.calculate_brightness(hex_color);
        brightness > 0.7 // Too light for light theme
    }
    
    fn calculate_brightness(&self, hex_color: &str) -> f32 {
        let hex = hex_color.trim_start_matches('#');
        if hex.len() != 6 {
            return 0.5; // Default to medium brightness for invalid colors
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32;
        
        // Perceived brightness formula
        (r * 0.299 + g * 0.587 + b * 0.114) / 255.0
    }
    
    fn detect_terminal_theme(&self) -> Option<bool> {
        // Try COLORFGBG first (most reliable when available)
        if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
            if let Some(bg) = colorfgbg.split(';').last() {
                if let Ok(bg_num) = bg.parse::<u8>() {
                    return Some(bg_num < 8); // Dark if background < 8
                }
            }
        }
        
        // Check other environment variables
        for var in &["TERM_THEME", "THEME", "COLOR_SCHEME"] {
            if let Ok(value) = std::env::var(var) {
                let lower = value.to_lowercase();
                if lower.contains("dark") {
                    return Some(true);
                } else if lower.contains("light") {
                    return Some(false);
                }
            }
        }
        
        None
    }
    
    pub fn get_contrasting_style(&self, hex_color: &str) -> Style {
        // Parse the color
        let color = self.parse_hex_color(hex_color).unwrap_or(Color::White);
        
        if self.is_color_problematic(hex_color) {
            // Problematic color - use neutral background with color as foreground
            let bg_color = match self.theme_mode {
                ThemeMode::Light => Color::Rgb(220, 220, 220), // Light grey for light theme
                _ => Color::Rgb(64, 64, 64), // Dark grey for dark theme
            };
            
            Style::default()
                .fg(color)
                .bg(bg_color)
        } else {
            // Good color - use as background with contrasting text
            let text_color = if self.calculate_brightness(hex_color) > 0.5 {
                Color::Black // Dark text on light background
            } else {
                Color::White // Light text on dark background
            };
            
            Style::default()
                .fg(text_color)
                .bg(color)
        }
    }
    
    fn parse_hex_color(&self, hex_color: &str) -> Option<Color> {
        let hex = hex_color.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        
        Some(Color::Rgb(r, g, b))
    }
}

impl Default for ColorHelper {
    fn default() -> Self {
        Self::new(ThemeMode::Auto)
    }
}