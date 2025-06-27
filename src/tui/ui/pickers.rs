// Picker rendering (project picker, filter picker)

use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear, Wrap};
use ratatui::text::{Line, Span};
use super::hex_to_color;

pub fn draw_project_picker_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = (area.height as f32 * 0.7) as u16;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(3),   // List
        ])
        .split(modal_area);
    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Filter Projects (type to search)")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Magenta));
    let input_paragraph = Paragraph::new(app.project_picker_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);
    // Project list
    let mut project_lines = Vec::new();
    for (i, (pid, name)) in app.filtered_projects.iter().enumerate() {
        let is_selected = i == app.selected_project_picker_index;
        let color = if *pid == -1 {
            Color::Cyan
        } else {
            app.project_colors.get(pid).map(|hex| hex_to_color(hex)).unwrap_or(Color::White)
        };
        let mut style = Style::default().fg(color);
        if is_selected {
            style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
        }
        project_lines.push(Line::from(vec![Span::styled(name, style)]));
    }
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title("Select Project (Enter to confirm, Esc to cancel)")
        .title_alignment(Alignment::Center);
    let list_paragraph = Paragraph::new(project_lines)
        .block(list_block)
        .wrap(Wrap { trim: false });
    f.render_widget(list_paragraph, modal_chunks[1]);
    // Position cursor in input
    let cursor_x = modal_chunks[0].x + 1 + app.project_picker_input.len() as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
}

pub fn draw_filter_picker_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = (area.height as f32 * 0.7) as u16;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Min(3),   // List
        ])
        .split(modal_area);
    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Filter Saved Views (type to search)")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Magenta));
    let input_paragraph = Paragraph::new(app.filter_picker_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);
    // Filter list
    let mut filter_lines = Vec::new();
    for (i, (_, title)) in app.filtered_filters.iter().enumerate() {
        let is_selected = i == app.selected_filter_picker_index;
        let mut style = Style::default().fg(Color::Cyan);
        if is_selected {
            style = style.add_modifier(Modifier::REVERSED | Modifier::BOLD);
        }
        filter_lines.push(Line::from(vec![Span::styled(title, style)]));
    }
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title("Select Saved Filter (Enter to confirm, Esc to cancel)")
        .title_alignment(Alignment::Center);
    let list_paragraph = Paragraph::new(filter_lines)
        .block(list_block)
        .wrap(Wrap { trim: false });
    f.render_widget(list_paragraph, modal_chunks[1]);
    // Position cursor in input
    let cursor_x = modal_chunks[0].x + 1 + app.filter_picker_input.len() as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
}
