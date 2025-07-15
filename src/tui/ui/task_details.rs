// Task details pane rendering

use crate::tui::app::state::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Wrap};
use ratatui::text::{Line, Span};
use chrono::Datelike;
use super::hex_to_color;

pub fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    let selected_task = app.get_selected_task();
    let details = if let Some(task) = selected_task {
        let project_name = app.project_map.get(&(task.project_id as i64))
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let mut labels_line_spans = vec![Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD))];
        if let Some(labels) = &task.labels {
            for (i, label) in labels.iter().enumerate() {
                let color = hex_to_color(label.hex_color.as_deref().unwrap_or(""));
                labels_line_spans.push(Span::styled(&label.title, Style::default().fg(color)));
                if i < labels.len() - 1 {
                    labels_line_spans.push(Span::raw(", "));
                }
            }
        }
        let mut details_lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.title)
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Project: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(project_name)
            ]),
            Line::from(""),
            Line::from(labels_line_spans),
            Line::from(""),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if task.done { "Completed" } else { "Pending" })
            ]),
            Line::from(""),
        ];
        if let Some(due_date) = &task.due_date {
            if due_date.year() > 1900 {
                details_lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(due_date.format("%Y-%m-%d %H:%M").to_string())
                ]));
                details_lines.push(Line::from(""));
            }
        }
        if let Some(priority) = task.priority {
            if priority > 0 {
                let priority_color = match priority {
                    5 => Color::Red,
                    4 => Color::Rgb(255, 165, 0),
                    3 => Color::Yellow,
                    2 => Color::Blue,
                    1 => Color::Magenta,
                    _ => Color::White,
                };
                details_lines.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(" ", Style::default().fg(priority_color)),
                    Span::raw(format!(" !{}", priority))
                ]));
                details_lines.push(Line::from(""));
            }
        }
        if task.is_favorite {
            details_lines.push(Line::from(vec![
                Span::styled("Starred: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default().fg(Color::Yellow)),
                Span::raw(" Yes")
            ]));
            details_lines.push(Line::from(""));
        }
        details_lines.push(Line::from(vec![
            Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(task.id.to_string())
        ]));
        details_lines
    } else {
        vec![Line::from("No task selected")]
    };
    let paragraph = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Task Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
