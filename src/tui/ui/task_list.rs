// Task list rendering

use crate::tui::app::state::App;
use crate::config::{TaskColumn, TableColumn};
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Table, Row, Cell, Block, Borders};
use ratatui::layout::Constraint;
use super::hex_to_color;
use chrono::{DateTime, Utc, Local};
use ratatui::text::{Line, Span};

fn format_date(date: &Option<DateTime<Utc>>) -> String {
    match date {
        Some(dt) => {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            local.format("%m/%d/%y").to_string()
        }
        None => "-".to_string(),
    }
}

fn format_date_relative(date: &Option<DateTime<Utc>>) -> (String, Color) {
    match date {
        Some(dt) => {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            let now = Local::now();
            let diff = local.signed_duration_since(now).num_days();
            
            let formatted = if diff == 0 {
                "Today".to_string()
            } else if diff == 1 {
                "Tomorrow".to_string()
            } else if diff == -1 {
                "Yesterday".to_string()
            } else if diff > 0 && diff <= 7 {
                format!("{}d", diff)
            } else if diff < 0 && diff >= -7 {
                format!("{}d ago", -diff)
            } else {
                local.format("%m/%d/%y").to_string()
            };
            
            let color = if diff < 0 {
                Color::Red // Overdue
            } else if diff == 0 {
                Color::Yellow // Due today
            } else if diff <= 3 {
                Color::Cyan // Due soon
            } else {
                Color::White // Normal
            };
            
            (formatted, color)
        }
        None => ("-".to_string(), Color::DarkGray),
    }
}

// Helper function to calculate optimal column widths
fn calculate_column_widths(
    columns: &[&TableColumn], 
    _tasks: &[crate::vikunja::models::Task], 
    _app: &App,
    available_width: u16
) -> Vec<u16> {
    let mut widths = Vec::new();
    let mut total_min_width = 0u16;
    let mut flexible_columns = 0;

    // First pass: calculate minimum widths and count flexible columns
    for column in columns {
        let min_width = column.min_width.unwrap_or(8);
        total_min_width += min_width;
        widths.push(min_width);
        
        if column.max_width.is_none() {
            flexible_columns += 1;
        }
    }

    // If we have more space than minimum, distribute it
    if available_width > total_min_width && flexible_columns > 0 {
        let extra_space = available_width - total_min_width;
        let space_per_flexible = extra_space / flexible_columns;
        
        for (i, column) in columns.iter().enumerate() {
            if column.max_width.is_none() {
                widths[i] += space_per_flexible;
            } else if let Some(max_width) = column.max_width {
                // For fixed-max columns, give them up to their max
                let current_width = widths[i];
                let can_grow = max_width.saturating_sub(current_width);
                let grow_amount = std::cmp::min(can_grow, space_per_flexible);
                widths[i] += grow_amount;
            }
        }
    }

    // Ensure no column exceeds its max width
    for (i, column) in columns.iter().enumerate() {
        if let Some(max_width) = column.max_width {
            widths[i] = std::cmp::min(widths[i], max_width);
        }
    }

    widths
}

// Helper function to wrap text if needed - returns owned strings
fn wrap_text_for_cell(text: &str, width: u16, should_wrap: bool) -> String {
    if !should_wrap || width == 0 {
        return text.to_string();
    }

    let width = width as usize;
    let mut result = String::new();
    
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        
        if line.len() <= width {
            result.push_str(line);
        } else {
            // Simple word wrapping
            let mut current_line = String::new();
            let mut first_word_in_line = true;
            
            for word in line.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                    first_word_in_line = false;
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    if !first_word_in_line {
                        result.push_str(&current_line);
                        result.push('\n');
                    }
                    current_line = word.to_string();
                    first_word_in_line = false;
                }
            }
            if !current_line.is_empty() {
                result.push_str(&current_line);
            }
        }
    }
    
    result
}

// Enhanced cell creation that supports wrapping
fn create_wrapped_cell_for_column<'a>(
    task: &'a crate::vikunja::models::Task, 
    column: &TableColumn, 
    app: &'a App,
    width: u16
) -> Cell<'a> {
    // Debug which column is being rendered
    
    let should_wrap = column.wrap_text.unwrap_or(false);
    
    match &column.column_type {
        TaskColumn::Title => {
            use ratatui::text::{Span, Line};
            let mut spans = Vec::new();
            if task.done {
                spans.push(Span::raw("✓ "));
            }
            if task.is_favorite {
                spans.push(Span::raw("\u{f005} "));
            }
            if let Some(p) = task.priority {
                if p >= 1 && p <= 5 {
                    let color = match p {
                        5 => Color::Red,
                        4 => Color::Rgb(255, 165, 0),
                        3 => Color::Yellow,
                        2 => Color::Blue,
                        1 => Color::Magenta,
                        _ => Color::White,
                    };
                    spans.push(Span::styled("\u{f024} ", Style::default().fg(color)));
                }
            }
            // Add relation indicators - DISABLED: Incomplete feature
            // if let Some(indicator) = app.get_task_relation_indicator(task) {
            //     spans.push(Span::raw(indicator));
            //     spans.push(Span::raw(" "));
            // }
            spans.push(Span::raw(&task.title));
            let line = Line::from(spans);
            let mut cell = Cell::from(line);
            if task.done {
                cell = cell.style(Style::default().add_modifier(Modifier::CROSSED_OUT).fg(Color::DarkGray));
            }
            cell
        }
        TaskColumn::Project => {
            let project_name = app.project_map.get(&task.project_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            let project_color = app.project_colors.get(&task.project_id)
                .and_then(|hex| Some(hex_to_color(hex.as_str())))
                .unwrap_or(Color::White);
            
            let truncated = if project_name.len() > width as usize {
                format!("{}…", &project_name[..width.saturating_sub(1) as usize])
            } else {
                project_name
            };
            
            Cell::from(truncated).style(Style::default().fg(project_color))
        }
        TaskColumn::Labels => {
            if let Some(labels) = &task.labels {
                let mut spans = Vec::new();
                for (i, label) in labels.iter().enumerate() {
                    let color = app.label_colors.get(&label.id)
                        .and_then(|hex| Some(hex_to_color(hex.as_str())))
                        .unwrap_or(ratatui::style::Color::Gray);
                    spans.push(Span::styled(
                        label.title.clone(),
                        Style::default().fg(color),
                    ));
                    if i < labels.len() - 1 {
                        spans.push(Span::raw(", "));
                    }
                }
                if should_wrap {
                    // Wrapping for colored spans is non-trivial; fallback to no wrap for now
                    Cell::from(Line::from(spans))
                } else {
                    Cell::from(Line::from(spans))
                }
            } else {
                Cell::from("")
            }
        }
        TaskColumn::DueDate => {
            let (formatted, color) = format_date_relative(&task.due_date);
            Cell::from(formatted).style(Style::default().fg(color))
        }
        TaskColumn::StartDate => {
            let formatted = format_date(&task.start_date);
            Cell::from(formatted).style(Style::default().fg(Color::Cyan))
        }
        TaskColumn::Priority => {
            
            match task.priority {
                Some(p) if p >= 1 && p <= 5 => {
                    // Nerd Font flag icon:  (U+F024)
                    let flag_icon = "\u{f024} ";
                    let color = match p {
                        5 => Color::Red,                // Highest priority
                        4 => Color::Rgb(255, 165, 0),   // High priority
                        3 => Color::Yellow,             // Medium priority
                        2 => Color::Blue,               // Low priority
                        1 => Color::Magenta,            // Lowest priority
                        _ => Color::White,              // Should never happen
                    };
                    Cell::from(format!("{}{}", flag_icon, p)).style(Style::default().fg(color))
                }
                _ => {
                    Cell::from("-")
                }
            }
        }
        TaskColumn::Status => {
            if task.done {
                Cell::from("Done").style(Style::default().fg(Color::Green))
            } else {
                Cell::from("Open").style(Style::default().fg(Color::White))
            }
        }
        TaskColumn::Assignees => {
            let assignees = task.assignees.as_ref()
                .map(|assignees| {
                    assignees.iter()
                        .map(|a| a.username.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string());
                
            if should_wrap {
                let wrapped = wrap_text_for_cell(&assignees, width, true);
                Cell::from(wrapped)
            } else {
                let truncated = if assignees.len() > width as usize {
                    format!("{}…", &assignees[..width.saturating_sub(1) as usize])
                } else {
                    assignees
                };
                Cell::from(truncated)
            }
        }
        TaskColumn::Created => {
            // Note: created is a string in the model, would need similar parsing
            Cell::from(task.created.as_ref().map(|_| "N/A").unwrap_or("-"))
        }
        TaskColumn::Updated => {
            // Note: updated is a string in the model, would need similar parsing  
            Cell::from(task.updated.as_ref().map(|_| "N/A").unwrap_or("-"))
        }
    }
}



pub fn draw_tasks_table(f: &mut Frame, app: &App, area: Rect) {
    let columns = app.get_current_layout_columns();
    
    let enabled_columns: Vec<&TableColumn> = columns.iter().filter(|c| c.enabled).collect();
    
    // Check if Priority column is enabled
    let _has_priority_column = enabled_columns.iter().any(|c| matches!(c.column_type, TaskColumn::Priority));
    // ...existing code...
    
    // Calculate optimal column widths
    let available_width = area.width.saturating_sub(enabled_columns.len() as u16 + 1); // Account for borders
    let column_widths = calculate_column_widths(&enabled_columns, &app.tasks, app, available_width);
    
    let header_cells: Vec<Cell> = enabled_columns.iter()
        .map(|col| Cell::from(col.name.as_str()).style(Style::default().fg(Color::Red)))
        .collect();
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Create constraints based on calculated widths
    let constraints: Vec<Constraint> = column_widths.iter()
        .map(|&width| Constraint::Length(width))
        .collect();

    // Calculate how many rows fit (minus header and margin)
    let total_height = area.height as usize;
    // 1 for header, 1 for header bottom margin, 1 for border, 1 for extra padding
    let visible_rows = if total_height > 4 { total_height - 4 } else { 1 };
    let num_tasks = app.tasks.len();
    let selected = app.selected_task_index;
    
    // Debug task priorities
    // ...existing code...

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
            // Create cells for each enabled column with proper width and wrapping
            let cells: Vec<Cell> = enabled_columns.iter()
                .zip(column_widths.iter())
                .map(|(col, &width)| create_wrapped_cell_for_column(task, col, app, width))
                .collect();
            
            let mut row = Row::new(cells);
            
            // Handle flashing effect
            let mut flash_bg = None;
            if let (Some(flash_id), Some(start_time)) = (app.flash_task_id, app.flash_start) {
                if task.id == flash_id {
                    let elapsed = Local::now().signed_duration_since(start_time).num_milliseconds() as u64;
                    let cycle = (elapsed / 50) as u8;
                    if usize::from(cycle) < app.flash_cycle_max {
                        let project_color = app.project_colors.get(&task.project_id)
                            .map(|hex| hex_to_color(hex))
                            .unwrap_or(Color::White);
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
            
            // Apply selection, flash, and alternating row styling
            if i == app.selected_task_index {
                // Selected row takes priority over alternating colors
                if let Some(bg) = flash_bg {
                    row = row.style(Style::default().bg(bg).add_modifier(Modifier::BOLD));
                } else {
                    row = row.style(Style::default().bg(Color::DarkGray));
                }
            } else if let Some(bg) = flash_bg {
                // Flash effect takes priority over alternating colors
                row = row.style(Style::default().bg(bg).add_modifier(Modifier::BOLD));
            } else {
                // Apply alternating row highlighting for easier scanning
                if i % 2 == 1 {
                    // Every other row gets a subtle background
                    row = row.style(Style::default().bg(Color::Rgb(40, 40, 50)));
                }
            }
            row
        });
    
    // Build comprehensive title with filter and project information
    let mut title = format!("Tasks ({})", app.get_filter_display_name());
    
    // Add project information if a specific project is selected
    if app.current_project_id.is_some() {
        let project_name = app.get_current_project_name();
        title = format!("Tasks ({}) - Project: {}", app.get_filter_display_name(), project_name);
    }
    // Add project override information if a filter has overridden the default project
    else if app.active_project_override.is_some() {
        let active_project = app.get_active_default_project();
        title = format!("Tasks ({}) - Default Project: {}", app.get_filter_display_name(), active_project);
    }
    
    let table = Table::new(rows, constraints)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));
    
    f.render_widget(table, area);
}

