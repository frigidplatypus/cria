use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::vikunja::models::Attachment;
use crate::tui::ui::attachment_viewer::AttachmentViewer;

use std::path::PathBuf;

/// Modal for viewing and managing task attachments
pub struct AttachmentModal {
    pub viewer: AttachmentViewer,
    pub task_title: String,
    pub task_id: i64,
    pub download_path: Option<PathBuf>,
    pub upload_path: Option<PathBuf>,
    pub operation_in_progress: bool,
    pub operation_message: String,
}

impl AttachmentModal {
    pub fn new(attachments: Vec<Attachment>, task_title: String, task_id: i64) -> Self {
        Self {
            viewer: AttachmentViewer::new(attachments),
            task_title,
            task_id,
            download_path: None,
            upload_path: None,
            operation_in_progress: false,
            operation_message: String::new(),
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
            .title(format!("📎 Attachments - {}", self.task_title))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let info_text = if self.viewer.attachments.is_empty() {
            "No attachments found for this task".to_string()
        } else {
            format!("{} attachment(s)", self.viewer.attachments.len())
        };

        let text = Paragraph::new(info_text)
            .block(block)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(text, area);
    }

    fn draw_content(&self, f: &mut Frame, area: Rect) {
        self.viewer.draw(f, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));

        let mut help_lines = Vec::new();
        
        // Show operation status if in progress
        if self.operation_in_progress {
            help_lines.push(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Yellow)),
                Span::styled(&self.operation_message, Style::default().fg(Color::Cyan)),
            ]));
            help_lines.push(Line::raw(""));
        }
        
        if !self.viewer.attachments.is_empty() {
            help_lines.push(Line::from(vec![
                Span::styled("↑/↓ ", Style::default().fg(Color::Yellow)),
                Span::styled("Navigate", Style::default().fg(Color::Gray)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("d ", Style::default().fg(Color::Yellow)),
                Span::styled("Download", Style::default().fg(Color::Gray)),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled("r ", Style::default().fg(Color::Yellow)),
                Span::styled("Remove", Style::default().fg(Color::Gray)),
            ]));
        }
        
        help_lines.push(Line::from(vec![
            Span::styled("u ", Style::default().fg(Color::Yellow)),
            Span::styled("Upload new file", Style::default().fg(Color::Gray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("q/ESC ", Style::default().fg(Color::Yellow)),
            Span::styled("Close", Style::default().fg(Color::Gray)),
        ]));

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

    pub fn handle_key(&mut self, key: char) -> AttachmentModalAction {
        match key {
            'j' | 'J' | ' ' => {
                self.viewer.next_attachment();
                AttachmentModalAction::None
            }
            'k' | 'K' => {
                self.viewer.previous_attachment();
                AttachmentModalAction::None
            }
            'd' | 'D' => {
                if let Some(attachment) = self.viewer.get_selected_attachment() {
                    AttachmentModalAction::Download(attachment.clone())
                } else {
                    AttachmentModalAction::None
                }
            }
            'r' | 'R' => {
                if let Some(attachment) = self.viewer.get_selected_attachment() {
                    AttachmentModalAction::Remove(attachment.clone())
                } else {
                    AttachmentModalAction::None
                }
            }
            'u' | 'U' => AttachmentModalAction::Upload,
            'q' | 'Q' | '\x1b' => AttachmentModalAction::Close, // ESC key
            _ => AttachmentModalAction::None,
        }
    }
}

/// Actions that can be performed in the attachment modal
#[derive(Debug, Clone)]
pub enum AttachmentModalAction {
    None,
    Close,
    Download(Attachment),
    Remove(Attachment),
    Upload,
}
 