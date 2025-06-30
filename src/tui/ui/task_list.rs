// Task list rendering

use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Table, Row, Cell, Block, Borders};
use ratatui::text::{Line, Span};
use super::hex_to_color;

pub fn draw_tasks_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Title", "Project", "Labels"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Calculate how many rows fit (minus header and margin)
    let total_height = area.height as usize;
    // 1 for header, 1 for header bottom margin, 1 for border, 1 for extra padding
    let visible_rows = if total_height > 4 { total_height - 4 } else { 1 };
    let num_tasks = app.tasks.len();
    let selected = app.selected_task_index;

    // Add a buffer at the bottom to always show the last N tasks when near the end
    let bottom_buffer = 3; // Number of tasks at the bottom to always keep visible
    let mut start = 0;
    let mut end = num_tasks;
    if num_tasks > visible_rows {
        if selected >= num_tasks.saturating_sub(bottom_buffer) {
            start = num_tasks.saturating_sub(visible_rows);
            end = num_tasks;
        } else if selected < visible_rows / 2 {
            start = 0;
            end = visible_rows;
        } else {
            start = selected + 1 - visible_rows / 2;
            if start + visible_rows > num_tasks {
                start = num_tasks - visible_rows;
            }
            end = start + visible_rows;
        }
    }

    // DEBUG: Print viewport and window info to debug log (comment out after confirming fix)
    // crate::debug::debug_log(&format!(
    //     "[TASKS] area.height={} visible_rows={} num_tasks={} selected={} start={} end={}",
    //     area.height, visible_rows, num_tasks, selected, start, end
    // ));

    let rows = app.tasks.iter().enumerate()
        .skip(start)
        .take(end - start)
        .map(|(i, task)| {
            let project_name = app.project_map.get(&task.project_id)
                .cloned()
                .unwrap_or_else(|| "Unknown Project".to_string());
            let project_color = app.project_colors.get(&task.project_id)
                .map(|hex| hex_to_color(hex))
                .unwrap_or(Color::White);
            let labels_line = task.labels.as_ref()
                .map(|labels| {
                    let spans: Vec<Span> = labels.iter()
                        .enumerate()
                        .flat_map(|(j, label)| {
                            let color = hex_to_color(&label.hex_color);
                            let mut spans = vec![Span::styled(&label.title, Style::default().fg(color))];
                            if j < labels.len() - 1 {
                                spans.push(Span::raw(", "));
                            }
                            spans
                        })
                        .collect();
                    Line::from(spans)
                })
                .unwrap_or_else(|| Line::raw(""));
            let mut title_spans = get_task_icons(task);
            if task.done {
                title_spans.push(Span::styled(&task.title, Style::default().add_modifier(Modifier::CROSSED_OUT).fg(Color::DarkGray)));
            } else {
                title_spans.push(Span::raw(&task.title));
            }
            let title_cell = Cell::from(Line::from(title_spans));
            let project_cell = Cell::from(project_name).style(Style::default().fg(project_color));
            let labels_cell = Cell::from(labels_line);
            let mut row = Row::new(vec![title_cell, project_cell, labels_cell]);
            let mut flash_bg = None;
            if let (Some(flash_id), Some(start_time)) = (app.flash_task_id, app.flash_start) {
                if task.id == flash_id {
                    let elapsed = start_time.elapsed().as_millis() as u64;
                    let cycle = (elapsed / 50) as u8;
                    if cycle < app.flash_cycle_max {
                        let base = match project_color {
                            Color::Rgb(r, g, b) => (r, g, b),
                            _ => (255, 255, 0),
                        };
                        let fade = if cycle % 2 == 0 {
                            (
                                (((base.0 as u16 + 255) / 2) as u8),
                                (((base.1 as u16 + 255) / 2) as u8),
                                (((base.2 as u16 + 255) / 2) as u8),
                            )
                        } else {
                            (base.0, base.1, base.2)
                        };
                        flash_bg = Some(Color::Rgb(fade.0, fade.1, fade.2));
                    }
                }
            }
            // Adjust for visible window
            let _visible_index = i - start;
            if i == app.selected_task_index {
                if let Some(bg) = flash_bg {
                    row = row.style(Style::default().bg(bg).add_modifier(Modifier::BOLD));
                } else {
                    row = row.style(Style::default().bg(Color::DarkGray));
                }
            } else if let Some(bg) = flash_bg {
                row = row.style(Style::default().bg(bg).add_modifier(Modifier::BOLD));
            }
            row
        });
    let table = Table::new(rows, &[Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!("Tasks ({})", app.get_filter_display_name())));
    f.render_widget(table, area);
}

pub fn get_task_icons(task: &crate::vikunja::models::Task) -> Vec<Span<'_>> {
    let mut icons = Vec::new();
    if task.done {
        icons.push(Span::styled("\u{f00c} ", Style::default().fg(Color::Green)));
    }
    if task.is_favorite {
        icons.push(Span::styled("\u{f005} ", Style::default().fg(Color::Yellow)));
    }
    if let Some(priority) = task.priority {
        let (icon, color) = match priority {
            5 => ("\u{f024} ", Color::Red),
            4 => ("\u{f024} ", Color::Rgb(255, 165, 0)),
            3 => ("\u{f024} ", Color::Yellow),
            2 => ("\u{f024} ", Color::Blue),
            1 => ("\u{f024} ", Color::Magenta),
            _ => ("", Color::White),
        };
        if !icon.is_empty() {
            icons.push(Span::styled(icon, Style::default().fg(color)));
        }
    }
    icons
}
