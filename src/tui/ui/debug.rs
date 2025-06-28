use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::text::{Line, Span};
use ratatui::style::{Style, Color};

pub fn draw_debug_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, f.size());
    let messages: Vec<Line> = app.debug_messages.iter().rev().take(30).map(|(dt, msg)| {
        Line::from(vec![Span::styled(
            format!("{}: {}", dt.format("%H:%M:%S"), msg),
            Style::default().fg(Color::Gray)
        )])
    }).collect();
    let block = Block::default().title("Debug Log").borders(Borders::ALL);
    let para = Paragraph::new(messages).block(block);
    f.render_widget(Clear, area); // Clear the area first
    f.render_widget(para, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let vertical = popup_layout[1];
    let horizontal_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical);
    horizontal_layout[1]
}
