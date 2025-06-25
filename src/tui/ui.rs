use crate::tui::app::App;
use ratatui::{
    prelude::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    text::{Line, Span},
    Frame,
};

fn hex_to_color(hex: &str) -> Color {
    // Remove the # if present
    let hex = hex.trim_start_matches('#');
    
    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            return Color::Rgb(r, g, b);
        }
    }
    
    // Fallback to white if parsing fails
    Color::White
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    draw_tasks_table(f, app, chunks[0]);
}

fn draw_tasks_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Title", "Project", "Labels"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.tasks.iter().map(|task| {
        let project_name = app.project_map.get(&task.project_id)
            .cloned()
            .unwrap_or_else(|| "Unknown Project".to_string());
        
        // Get project color
        let project_color = app.project_colors.get(&task.project_id)
            .map(|hex| hex_to_color(hex))
            .unwrap_or(Color::White);
        
        // Create colored spans for each label
        let labels_line = task.labels.as_ref()
            .map(|labels| {
                let spans: Vec<Span> = labels.iter()
                    .enumerate()
                    .flat_map(|(i, label)| {
                        let color = hex_to_color(&label.hex_color);
                        let mut spans = vec![Span::styled(&label.title, Style::default().fg(color))];
                        
                        // Add comma and space after each label except the last one
                        if i < labels.len() - 1 {
                            spans.push(Span::raw(", "));
                        }
                        
                        spans
                    })
                    .collect();
                Line::from(spans)
            })
            .unwrap_or_else(|| Line::raw(""));

        // Create colored cells
        let title_cell = Cell::from(task.title.clone());
        let project_cell = Cell::from(project_name).style(Style::default().fg(project_color));
        let labels_cell = Cell::from(labels_line);

        Row::new(vec![title_cell, project_cell, labels_cell])
    });

    let table = Table::new(rows, &[Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Tasks"));

    f.render_widget(table, area);
}
