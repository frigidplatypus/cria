use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;
use tokio::fs;

/// File picker modal for selecting files to upload
pub struct FilePickerModal {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub show_hidden: bool,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_file: bool,
    pub size: Option<u64>,
}

#[derive(Debug)]
pub enum FilePickerAction {
    Select(PathBuf),
    Cancel,
    Navigate(PathBuf),
    ToggleHidden,
    None,
}

impl FilePickerModal {
    pub fn new(initial_path: Option<PathBuf>) -> Self {
        let current_path = initial_path.unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        
        Self {
            current_path,
            entries: Vec::new(),
            selected_index: 0,
            show_hidden: false,
        }
    }

    pub async fn refresh_entries(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = Vec::new();
        
        // Add parent directory entry if not at root
        if self.current_path.parent().is_some() {
            entries.push(FileEntry {
                name: "..".to_string(),
                path: self.current_path.parent().unwrap().to_path_buf(),
                is_dir: true,
                is_file: false,
                size: None,
            });
        }
        
        // Read directory entries
        let mut read_dir = fs::read_dir(&self.current_path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("???")
                .to_string();
            
            // Skip hidden files unless show_hidden is true
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }
            
            let metadata = entry.metadata().await?;
            let is_dir = metadata.is_dir();
            let is_file = metadata.is_file();
            let size = if is_file { Some(metadata.len()) } else { None };
            
            entries.push(FileEntry {
                name,
                path,
                is_dir,
                is_file,
                size,
            });
        }
        
        // Sort: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                // Directories first
                b.is_dir.cmp(&a.is_dir)
            } else {
                // Then alphabetically
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });
        
        self.entries = entries;
        
        // Ensure selected_index is valid
        if !self.entries.is_empty() && self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len() - 1;
        }
        
        Ok(())
    }
    
    /// Synchronously load directory entries (for initial display)
    pub fn refresh_entries_sync(&mut self) {
        let mut entries = Vec::new();
        // Parent directory
        if let Some(parent) = self.current_path.parent() {
            entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                is_file: false,
                size: None,
            });
        }
        // Read directory entries synchronously
        if let Ok(read_dir) = std::fs::read_dir(&self.current_path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("???")
                    .to_string();
                // Skip hidden unless enabled
                if !self.show_hidden && name.starts_with('.') {
                    continue;
                }
                if let Ok(metadata) = entry.metadata() {
                    let is_dir = metadata.is_dir();
                    let is_file = metadata.is_file();
                    let size = if is_file { Some(metadata.len()) } else { None };
                    entries.push(FileEntry { name, path, is_dir, is_file, size });
                }
            }
        }
        // Sort: dirs first, then files, alphabetically
        entries.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });
        self.entries = entries;
        // Adjust selected index
        if !self.entries.is_empty() && self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len() - 1;
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        // Clear the entire screen first
        let clear_area = Rect::new(0, 0, area.width, area.height);
        f.render_widget(ratatui::widgets::Clear, clear_area);
        
        // Create a centered modal area
        let modal_area = self.center_modal(area, 80, 80);
        
        // Draw the modal background
        let background = Block::default()
            .style(Style::default().bg(Color::Black));
        f.render_widget(background, modal_area);

        // Split modal into header, content, and footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ])
            .split(modal_area);

        self.draw_header(f, chunks[0]);
        self.draw_content(f, chunks[1]);
        self.draw_footer(f, chunks[2]);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("📁 File Picker")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Blue));

        let path_text = self.current_path.to_string_lossy();
        let text = Paragraph::new(format!("Current: {}", path_text))
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(text, area);
    }

    fn draw_content(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));

        if self.entries.is_empty() {
            let text = Paragraph::new("No files found")
                .block(block)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(text, area);
            return;
        }

        let items: Vec<ListItem> = self.entries.iter().enumerate().map(|(i, entry)| {
            let is_selected = i == self.selected_index;
            
            let icon = if entry.is_dir {
                "📁"
            } else {
                "📄"
            };
            
            let name_style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let size_text = if let Some(size) = entry.size {
                format!(" ({})", self.format_size(size))
            } else {
                String::new()
            };
            
            let text = format!("{} {}{}", icon, entry.name, size_text);
            ListItem::new(text).style(name_style)
        }).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Blue).add_modifier(ratatui::style::Modifier::BOLD));
        
        f.render_stateful_widget(list, area, &mut ratatui::widgets::ListState::default().with_selected(Some(self.selected_index)));
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));

        let help_lines = vec![
            Line::from(vec![
                Span::styled("↑/↓ ", Style::default().fg(Color::Yellow)),
                Span::styled("Navigate", Style::default().fg(Color::Gray)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter ", Style::default().fg(Color::Yellow)),
                Span::styled("Select file/Open dir", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("h ", Style::default().fg(Color::Yellow)),
                Span::styled("Toggle hidden files", Style::default().fg(Color::Gray)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("q/ESC ", Style::default().fg(Color::Yellow)),
                Span::styled("Cancel", Style::default().fg(Color::Gray)),
            ]),
        ];

        let text = Paragraph::new(help_lines)
            .block(block);
        f.render_widget(text, area);
    }

    fn center_modal(&self, area: Rect, width_percent: u16, height_percent: u16) -> Rect {
        let width = (area.width * width_percent) / 100;
        let height = (area.height * height_percent) / 100;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        Rect::new(area.x + x, area.y + y, width, height)
    }

    pub fn handle_key(&mut self, key: char) -> FilePickerAction {
        match key {
            'j' | 'J' => {
                if self.selected_index < self.entries.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                FilePickerAction::None
            }
            'k' | 'K' => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                FilePickerAction::None
            }
            'h' | 'H' => {
                self.show_hidden = !self.show_hidden;
                FilePickerAction::ToggleHidden
            }
            'q' | 'Q' | '\x1b' => FilePickerAction::Cancel, // ESC key
            _ => FilePickerAction::None,
        }
    }

    pub fn handle_enter(&mut self) -> FilePickerAction {
        if self.entries.is_empty() {
            return FilePickerAction::None;
        }
        
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir {
                // Navigate to directory
                FilePickerAction::Navigate(entry.path.clone())
            } else if entry.is_file {
                // Select file
                FilePickerAction::Select(entry.path.clone())
            } else {
                FilePickerAction::None
            }
        } else {
            FilePickerAction::None
        }
    }

    fn format_size(&self, size: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        let size = size as f64;
        
        if size >= GB {
            format!("{:.1} GB", size / GB)
        } else if size >= MB {
            format!("{:.1} MB", size / MB)
        } else if size >= KB {
            format!("{:.1} KB", size / KB)
        } else {
            format!("{} B", size as u64)
        }
    }
} 