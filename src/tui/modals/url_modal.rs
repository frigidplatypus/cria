use ratatui::{
    layout::{Layout, Direction, Constraint, Rect, Alignment},
    style::{Color, Style, Modifier},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame,
};
use crate::url_utils::UrlWithContext;

/// URL selection modal for opening URLs found in tasks
pub struct UrlModal {
    pub urls: Vec<UrlWithContext>,
    pub selected_index: usize,
}

#[derive(Debug)]
pub enum UrlModalAction {
    OpenUrl(String),
    Cancel,
    None,
}

impl UrlModal {
    pub fn new(urls: Vec<UrlWithContext>) -> Self {
        Self {
            urls,
            selected_index: 0,
        }
    }
    
    pub fn handle_key(&mut self, key: char) -> UrlModalAction {
        let len = self.urls.len();
        match key {
            'j' => {
                if len == 0 {
                    // nothing to do
                } else if self.selected_index < len - 1 {
                    self.selected_index += 1;
                } else {
                    self.selected_index = 0;
                }
                UrlModalAction::None
            }
            'k' => {
                if len == 0 {
                    // nothing to do
                } else if self.selected_index > 0 {
                    self.selected_index -= 1;
                } else {
                    self.selected_index = len - 1;
                }
                UrlModalAction::None
            }
            _ => UrlModalAction::None,
        }
    }
    
    pub fn handle_enter(&self) -> UrlModalAction {
        if let Some(url_context) = self.urls.get(self.selected_index) {
            UrlModalAction::OpenUrl(url_context.url.clone())
        } else {
            UrlModalAction::Cancel
        }
    }
    
    pub fn handle_up(&mut self) {
        let len = self.urls.len();
        if len == 0 {
            // nothing to do
        } else if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = len - 1;
        }
    }

    pub fn handle_down(&mut self) {
        let len = self.urls.len();
        if len == 0 {
            // nothing to do
        } else if self.selected_index < len - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// Returns the currently selected URL as Option<&str>
    pub fn get_selected_url(&self) -> Option<&str> {
        self.urls.get(self.selected_index).map(|ctx| ctx.url.as_str())
    }
}

pub fn draw_url_modal(f: &mut Frame, modal: &UrlModal, area: Rect) {
    // Safe area check
    if area.width < 40 || area.height < 10 {
        let msg = Paragraph::new("Viewport too small for URL modal").style(Style::default().fg(Color::Red));
        f.render_widget(msg, area);
        return;
    }
    // ...existing code...
    let modal_width = std::cmp::min(80, area.width.saturating_sub(4));
    let modal_height = std::cmp::min(
        modal.urls.len() as u16 + 6,
        area.height.saturating_sub(4)
    );
    let modal_area = Rect {
        x: (area.width.saturating_sub(modal_width)) / 2,
        y: (area.height.saturating_sub(modal_height)) / 2,
        width: modal_width,
        height: modal_height,
    };
    f.render_widget(Clear, modal_area);
    let block = Block::default()
        .title("Open URL")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, modal_area);
    let inner_area = Rect {
        x: modal_area.x + 1,
        y: modal_area.y + 1,
        width: modal_area.width.saturating_sub(2),
        height: modal_area.height.saturating_sub(2),
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(inner_area);
    
    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("↑/k", Style::default().fg(Color::Yellow)),
            Span::raw(", "),
            Span::styled("↓/j", Style::default().fg(Color::Yellow)),
            Span::raw(" navigate • "),
            Span::styled("Enter", Style::default().fg(Color::Green)),
            Span::raw(" open • "),
            Span::styled("Esc", Style::default().fg(Color::Red)),
            Span::raw(" cancel"),
        ])
    ])
    .alignment(Alignment::Center);
    
    f.render_widget(instructions, chunks[0]);
    
    // URL list
    let items: Vec<ListItem> = modal.urls
        .iter()
        .enumerate()
        .map(|(i, url_ctx)| {
            let style = if i == modal.selected_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            // Truncate URL if too long for display
            let display_width = chunks[1].width.saturating_sub(4) as usize; // Account for padding
            let truncated_url = if url_ctx.url.len() > display_width {
                format!("{}...", &url_ctx.url[..display_width.saturating_sub(3)])
            } else {
                url_ctx.url.clone()
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(truncated_url, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("  from: {}", url_ctx.source),
                        Style::default().fg(Color::DarkGray)
                    ),
                ]),
            ]).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    
    f.render_widget(list, chunks[1]);
}
