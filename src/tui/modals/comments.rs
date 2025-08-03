use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::vikunja::models::{Comment, Attachment};
use crate::tui::ui::attachment_viewer::AttachmentViewer;

/// Enhanced modal for viewing and adding task comments with inline image previews
pub struct CommentsModal {
    pub comments: Vec<Comment>,
    pub attachments: Vec<Attachment>,
    pub input: String,
    pub cursor_position: usize,
    pub task_id: i64,
    pub scroll_offset: usize,
    pub selected_comment: usize,
    pub view_mode: CommentViewMode,
    pub attachment_viewer: Option<AttachmentViewer>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommentViewMode {
    List,
    Detail,
    AttachmentPreview,
}

/// Actions that can be performed in the comments modal
#[derive(Debug, Clone)]
pub enum CommentsModalAction {
    None,
    Close,
    Submit(String),
    ToggleMode,
    LoadAttachments(u64), // task_id for loading attachments
}

impl CommentsModal {
    pub fn new(comments: Vec<Comment>, task_id: i64) -> Self {
        Self {
            comments,
            attachments: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            task_id,
            scroll_offset: 0,
            selected_comment: 0,
            view_mode: CommentViewMode::List,
            attachment_viewer: None,
        }
    }

    pub fn with_attachments(mut self, attachments: Vec<Attachment>) -> Self {
        self.attachments = attachments;
        if !self.attachments.is_empty() {
            self.attachment_viewer = Some(AttachmentViewer::new(self.attachments.clone()));
        }
        self
    }

    pub fn draw(&self, f: &mut Frame, area: Rect) {
        // Clear background
        f.render_widget(Clear, area);
        
        // Modal dimensions - larger for advanced view
        let width = (area.width * 85) / 100;
        let height = (area.height * 85) / 100;
        let x = (area.width - width) / 2 + area.x;
        let y = (area.height - height) / 2 + area.y;
        let modal_area = Rect::new(x, y, width, height);

        match self.view_mode {
            CommentViewMode::List => self.draw_list_view(f, modal_area),
            CommentViewMode::Detail => self.draw_detail_view(f, modal_area),
            CommentViewMode::AttachmentPreview => self.draw_attachment_view(f, modal_area),
        }
    }

    fn draw_list_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Header
                Constraint::Min(8),       // Comments list
                Constraint::Length(3),    // Input
                Constraint::Length(2),    // Help
            ])
            .split(area);

        // Header with mode indicators
        let header_text = format!(
            "💬 Advanced Comments ({}) - {} | Attachments: {}",
            self.comments.len(),
            match self.view_mode {
                CommentViewMode::List => "List View",
                CommentViewMode::Detail => "Detail View", 
                CommentViewMode::AttachmentPreview => "Attachments",
            },
            self.attachments.len()
        );
        
        let header = Block::default()
            .title(header_text)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        // Enhanced comments list with better formatting
        self.draw_enhanced_comments_list(f, chunks[1]);

        // Input
        self.draw_input_section(f, chunks[2]);

        // Help section
        self.draw_help_section(f, chunks[3]);
    }

    fn draw_detail_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Comments detail
                Constraint::Percentage(40), // Attachments or metadata
            ])
            .split(area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Header
                Constraint::Min(5),       // Selected comment detail
                Constraint::Length(3),    // Input
                Constraint::Length(2),    // Help
            ])
            .split(chunks[0]);

        // Header
        let header = Block::default()
            .title("💬 Comment Details")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, left_chunks[0]);

        // Selected comment detail
        self.draw_selected_comment_detail(f, left_chunks[1]);

        // Input
        self.draw_input_section(f, left_chunks[2]);

        // Help
        self.draw_help_section(f, left_chunks[3]);

        // Right side - attachments or metadata
        if !self.attachments.is_empty() {
            if let Some(ref viewer) = self.attachment_viewer {
                viewer.draw(f, chunks[1]);
            }
        } else {
            self.draw_comment_metadata(f, chunks[1]);
        }
    }

    fn draw_attachment_view(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),    // Header
                Constraint::Min(10),      // Attachment viewer
                Constraint::Length(3),    // Input
                Constraint::Length(2),    // Help
            ])
            .split(area);

        // Header
        let header = Block::default()
            .title("🖼️ Attachments & Images")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        f.render_widget(header, chunks[0]);

        // Attachment viewer
        if let Some(ref viewer) = self.attachment_viewer {
            viewer.draw(f, chunks[1]);
        } else {
            let no_attachments = Paragraph::new("No attachments available for this task")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(no_attachments, chunks[1]);
        }

        // Input
        self.draw_input_section(f, chunks[2]);

        // Help
        self.draw_help_section(f, chunks[3]);
    }

    fn draw_enhanced_comments_list(&self, f: &mut Frame, area: Rect) {
        if self.comments.is_empty() {
            let no_comments = Paragraph::new("No comments yet. Add the first comment below!")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            f.render_widget(no_comments, area);
            return;
        }

        let mut lines = Vec::new();
        
        for (i, comment) in self.comments.iter().enumerate() {
            let is_selected = i == self.selected_comment;
            
            // Author and timestamp line
            let author = comment.author.as_ref()
                .map(|u| u.username.clone())
                .unwrap_or_else(|| "unknown".to_string());
            
            let timestamp = comment.created.as_ref()
                .map(|t| format!(" • {}", t))
                .unwrap_or_default();

            let author_style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            };

            let timestamp_style = if is_selected {
                Style::default().fg(Color::Gray).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("👤 {}", author), author_style),
                Span::styled(timestamp, timestamp_style),
            ]));

            // Comment content
            let text = comment.comment.clone().unwrap_or_default();
            let content_lines = text.lines().collect::<Vec<&str>>();
            
            for (line_idx, line) in content_lines.iter().enumerate() {
                let content_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if line_idx == 0 { "💬 " } else { "   " };
                lines.push(Line::from(vec![
                    Span::styled(format!("{}{}", prefix, line), content_style)
                ]));
            }

            // Add spacing between comments
            lines.push(Line::from(""));
        }

        let comments_block = Block::default()
            .borders(Borders::ALL)
            .title("Comments")
            .style(Style::default().fg(Color::White));
        
        let comments_para = Paragraph::new(lines)
            .block(comments_block)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, 0));
        
        f.render_widget(comments_para, area);
    }

    fn draw_selected_comment_detail(&self, f: &mut Frame, area: Rect) {
        if self.comments.is_empty() {
            let no_comments = Paragraph::new("No comments to show")
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(no_comments, area);
            return;
        }

        let comment = &self.comments[self.selected_comment];
        let mut lines = Vec::new();

        // Author info
        if let Some(author) = &comment.author {
            lines.push(Line::from(vec![
                Span::styled("👤 Author: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    author.username.clone(),
                    Style::default().fg(Color::Yellow)
                ),
            ]));
            
            if let Some(name) = &author.name {
                lines.push(Line::from(vec![
                    Span::styled("   Name: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(name, Style::default().fg(Color::Cyan)),
                ]));
            }
        }

        // Timestamp
        if let Some(created) = &comment.created {
            lines.push(Line::from(vec![
                Span::styled("📅 Created: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(created, Style::default().fg(Color::Green)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("💬 Comment:", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        // Comment content with proper formatting
        let text = comment.comment.clone().unwrap_or_default();
        for line in text.lines() {
            lines.push(Line::from(line));
        }

        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title("Comment Detail")
            .style(Style::default().fg(Color::White));
        
        let detail_para = Paragraph::new(lines)
            .block(detail_block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(detail_para, area);
    }

    fn draw_comment_metadata(&self, f: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        
        lines.push(Line::from(vec![
            Span::styled("📊 Comments Overview", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        ]));
        lines.push(Line::from(""));
        
        lines.push(Line::from(vec![
            Span::styled("Total Comments: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(self.comments.len().to_string(), Style::default().fg(Color::Yellow)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Attachments: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(self.attachments.len().to_string(), Style::default().fg(Color::Green)),
        ]));

        // Show unique authors
        let mut authors = std::collections::HashSet::new();
        for comment in &self.comments {
            if let Some(author) = &comment.author {
                authors.insert(&author.username);
            }
        }
        
        lines.push(Line::from(vec![
            Span::styled("Contributors: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(authors.len().to_string(), Style::default().fg(Color::Magenta)),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from("🔄 Press Tab to switch views"));
        lines.push(Line::from("🖼️ Press 'a' for attachments"));

        let metadata_block = Block::default()
            .borders(Borders::ALL)
            .title("Metadata")
            .style(Style::default().fg(Color::White));
        
        let metadata_para = Paragraph::new(lines)
            .block(metadata_block)
            .wrap(Wrap { trim: true });
        
        f.render_widget(metadata_para, area);
    }

    fn draw_input_section(&self, f: &mut Frame, area: Rect) {
        let input_block = Block::default()
            .title("💭 New Comment")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));
        
        let input_para = Paragraph::new(self.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(input_para, area);
        
        // Set cursor
        let cursor_x = area.x + 1 + self.cursor_position as u16;
        let cursor_y = area.y + 1;
        f.set_cursor(cursor_x, cursor_y);
    }

    fn draw_help_section(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.view_mode {
            CommentViewMode::List => "Tab: Detail | A: Attachments | ↑↓: Select | Enter: Submit | Esc: Close",
            CommentViewMode::Detail => "Tab: List | A: Attachments | ↑↓: Select | Enter: Submit | Esc: Close",
            CommentViewMode::AttachmentPreview => "Tab: List | ↑↓: Select | D: Download | R: Remove | Esc: Close",
        };

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(help, area);
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> CommentsModalAction {
        match key.code {
            KeyCode::Char(c) if self.view_mode != CommentViewMode::AttachmentPreview => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                CommentsModalAction::None
            }
            KeyCode::Backspace if self.view_mode != CommentViewMode::AttachmentPreview => {
                if self.cursor_position > 0 && self.cursor_position <= self.input.len() {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                }
                CommentsModalAction::None
            }
            KeyCode::Enter if self.view_mode != CommentViewMode::AttachmentPreview => {
                let text = self.input.clone();
                CommentsModalAction::Submit(text)
            }
            KeyCode::Tab => {
                self.view_mode = match self.view_mode {
                    CommentViewMode::List => CommentViewMode::Detail,
                    CommentViewMode::Detail => CommentViewMode::List,
                    CommentViewMode::AttachmentPreview => CommentViewMode::List,
                };
                CommentsModalAction::None
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.view_mode = CommentViewMode::AttachmentPreview;
                CommentsModalAction::None
            }
            KeyCode::Up => {
                if !self.comments.is_empty() && self.selected_comment > 0 {
                    self.selected_comment -= 1;
                }
                CommentsModalAction::None
            }
            KeyCode::Down => {
                if !self.comments.is_empty() && self.selected_comment < self.comments.len() - 1 {
                    self.selected_comment += 1;
                }
                CommentsModalAction::None
            }
            KeyCode::PageUp => {
                if self.scroll_offset > 0 {
                    self.scroll_offset = self.scroll_offset.saturating_sub(5);
                }
                CommentsModalAction::None
            }
            KeyCode::PageDown => {
                self.scroll_offset += 5;
                CommentsModalAction::None
            }
            KeyCode::Esc => CommentsModalAction::Close,
            _ => CommentsModalAction::None,
        }
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn add_comment(&mut self, comment: Comment) {
        self.comments.push(comment);
    }
}
