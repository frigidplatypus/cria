use std::env;

/// Terminal capabilities detection
pub struct TerminalCapabilities {
    pub supports_images: bool,
    pub supports_unicode: bool,
    pub supports_colors: bool,
    pub terminal_type: String,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    pub fn detect() -> Self {
        let terminal_type = env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
        let supports_colors = Self::detect_color_support(&terminal_type);
        let supports_unicode = Self::detect_unicode_support();
        let supports_images = Self::detect_image_support(&terminal_type);

        Self {
            supports_images,
            supports_unicode,
            supports_colors,
            terminal_type,
        }
    }

    /// Detect if terminal supports colors
    fn detect_color_support(term_type: &str) -> bool {
        // Most modern terminals support colors
        !term_type.contains("dumb") && !term_type.contains("unknown")
    }

    /// Detect if terminal supports unicode
    fn detect_unicode_support() -> bool {
        // Check if we can output unicode characters
        if let Ok(lang) = env::var("LANG") {
            lang.contains("UTF-8") || lang.contains("utf8")
        } else {
            // Assume unicode support for modern terminals
            true
        }
    }

    /// Detect if terminal supports images
    fn detect_image_support(term_type: &str) -> bool {
        // Check for terminals that support images
        let image_supporting_terms = [
            "kitty",
            "wezterm", 
            "iTerm2",
            "terminology",
            "contour",
            "foot",
            "alacritty", // with image protocol
        ];

        // Check if terminal type matches known image-supporting terminals
        if image_supporting_terms.iter().any(|&term| term_type.contains(term)) {
            return true;
        }

        // Check for specific environment variables
        if env::var("KITTY_WINDOW_ID").is_ok() {
            return true; // Kitty terminal
        }

        if env::var("WEZTERM_PANE").is_ok() {
            return true; // WezTerm
        }

        // Check for iTerm2
        if term_type.contains("xterm") && env::var("ITERM_PROFILE").is_ok() {
            return true;
        }

        false
    }

    /// Check if a file is an image based on extension and content
    pub fn is_image_file(filename: &str, mime_type: Option<&str>) -> bool {
        let image_extensions = [
            "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tga"
        ];

        // Check file extension
        if let Some(ext) = filename.split('.').last() {
            if image_extensions.iter().any(|&img_ext| ext.eq_ignore_ascii_case(img_ext)) {
                return true;
            }
        }

        // Check MIME type if available
        if let Some(mime) = mime_type {
            if mime.starts_with("image/") {
                return true;
            }
        }

        false
    }

    /// Get a simple ASCII representation of an image file
    pub fn get_image_ascii_art(filename: &str) -> Option<String> {
        // Simple ASCII art based on file type
        let ext = filename.split('.').last().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Some("🖼️  [JPEG Image]".to_string()),
            "png" => Some("🖼️  [PNG Image]".to_string()),
            "gif" => Some("🎬 [GIF Image]".to_string()),
            "webp" => Some("🖼️  [WebP Image]".to_string()),
            "svg" => Some("🎨 [SVG Image]".to_string()),
            "pdf" => Some("📄 [PDF Document]".to_string()),
            "txt" => Some("📝 [Text File]".to_string()),
            "doc" | "docx" => Some("📄 [Word Document]".to_string()),
            "xls" | "xlsx" => Some("📊 [Excel Spreadsheet]".to_string()),
            "zip" | "rar" | "7z" => Some("📦 [Archive]".to_string()),
            _ => Some("📎 [File]".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_file_detection() {
        assert!(TerminalCapabilities::is_image_file("test.jpg", None));
        assert!(TerminalCapabilities::is_image_file("test.PNG", None));
        assert!(TerminalCapabilities::is_image_file("test.gif", Some("image/gif")));
        assert!(!TerminalCapabilities::is_image_file("test.txt", None));
        assert!(TerminalCapabilities::is_image_file("test.txt", Some("image/png")));
    }

    #[test]
    fn test_ascii_art_generation() {
        assert_eq!(
            TerminalCapabilities::get_image_ascii_art("test.jpg"),
            Some("🖼️  [JPEG Image]".to_string())
        );
        assert_eq!(
            TerminalCapabilities::get_image_ascii_art("test.pdf"),
            Some("📄 [PDF Document]".to_string())
        );
    }
} 