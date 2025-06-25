use crate::tui::app::App;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Wrap, Clear},
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
    let _main_layout = if app.show_debug_pane {
        // Three-pane layout: tasks | info | debug
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // Tasks
                Constraint::Percentage(30),  // Info
                Constraint::Percentage(30),  // Debug
            ])
            .split(f.size());
        
        draw_tasks_table(f, app, horizontal_chunks[0]);
        if app.show_info_pane {
            draw_task_details(f, app, horizontal_chunks[1]);
        }
        draw_debug_pane(f, app, horizontal_chunks[2]);
        
    } else if app.show_info_pane {
        // Two-pane layout: tasks | info
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(f.size());
        
        draw_tasks_table(f, app, chunks[0]);
        draw_task_details(f, app, chunks[1]);
    } else {
        // Single pane: just tasks
        draw_tasks_table(f, app, f.size());
    };

    // Draw modal on top if active
    if app.show_quick_add_modal {
        draw_quick_add_modal(f, app);
    }
}

fn draw_tasks_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Title", "Project", "Labels"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.tasks.iter().enumerate().map(|(i, task)| {
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
                    .flat_map(|(j, label)| {
                        let color = hex_to_color(&label.hex_color);
                        let mut spans = vec![Span::styled(&label.title, Style::default().fg(color))];
                        
                        // Add comma and space after each label except the last one
                        if j < labels.len() - 1 {
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

        let mut row = Row::new(vec![title_cell, project_cell, labels_cell]);
        
        // Highlight the selected row
        if i == app.selected_task_index {
            row = row.style(Style::default().bg(Color::DarkGray));
        }
        
        row
    });

    let table = Table::new(rows, &[Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Tasks"));

    f.render_widget(table, area);
}

fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    let selected_task = app.get_selected_task();
    
    let details = if let Some(task) = selected_task {
        let project_name = app.project_map.get(&(task.project_id as i64))
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
            
        // Create colored labels line for display
        let mut labels_line_spans = vec![Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD))];
        
        if let Some(labels) = &task.labels {
            for (i, label) in labels.iter().enumerate() {
                let color = hex_to_color(&label.hex_color);
                labels_line_spans.push(Span::styled(&label.title, Style::default().fg(color)));
                
                // Add comma and space after each label except the last one
                if i < labels.len() - 1 {
                    labels_line_spans.push(Span::raw(", "));
                }
            }
        }
        
        vec![
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
            Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(task.id.to_string())
            ]),
        ]
    } else {
        vec![Line::from("No task selected")]
    };

    let paragraph = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Task Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_quick_add_modal(f: &mut Frame, app: &App) {
    // Calculate centered modal area (80% width, 18 lines height for better visibility)
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 18;
    
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    
    let modal_area = Rect {
        x,
        y,
        width: modal_width,
        height: modal_height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Create the modal layout - two sections: input and help
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field with title
            Constraint::Min(14),   // Help text
        ])
        .split(modal_area);

    // Input field with title
    let input_text = app.get_quick_add_input();
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Quick Add Task")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    
    let input_paragraph = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);

    // Position cursor
    let cursor_x = modal_chunks[0].x + 1 + app.quick_add_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("Quick Add Magic Examples:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("• "),
            Span::styled("Buy groceries *shopping *urgent", Style::default().fg(Color::White)),
            Span::raw(" - adds labels")
        ]),
        Line::from(vec![
            Span::raw("• "),
            Span::styled("Review PR @john", Style::default().fg(Color::White)),
            Span::raw(" - assigns to user")
        ]),
        Line::from(vec![
            Span::raw("• "),
            Span::styled("Fix bug +work !3", Style::default().fg(Color::White)),
            Span::raw(" - sets project & priority")
        ]),
        Line::from(vec![
            Span::raw("• "),
            Span::styled("Call mom tomorrow at 2pm", Style::default().fg(Color::White)),
            Span::raw(" - sets due date")
        ]),
        Line::from(vec![
            Span::raw("• "),
            Span::styled("Team meeting every Monday", Style::default().fg(Color::White)),
            Span::raw(" - recurring task")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Syntax: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("*label ", Style::default().fg(Color::Red)),
            Span::styled("@user ", Style::default().fg(Color::Blue)),
            Span::styled("+project ", Style::default().fg(Color::Magenta)),
            Span::styled("!priority ", Style::default().fg(Color::Yellow)),
            Span::styled("dates", Style::default().fg(Color::Cyan))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" to create • "),
            Span::styled("Escape", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" to cancel")
        ]),
    ];

    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Help")
        .style(Style::default().fg(Color::Gray));
    
    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .wrap(Wrap { trim: true });
    f.render_widget(help_paragraph, modal_chunks[1]);
}

fn draw_debug_pane(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Debug (d: toggle, c: clear)")
        .style(Style::default().fg(Color::Cyan));

    // Show the last debug messages (most recent at bottom)
    let debug_lines: Vec<Line> = app.debug_messages
        .iter()
        .rev()  // Reverse to show newest at top
        .take(area.height.saturating_sub(3) as usize)  // Fit within the pane
        .rev()  // Reverse back to show chronological order
        .map(|(timestamp, msg)| {
            let time_str = timestamp.format("%H:%M:%S");
            let formatted_msg = format!("[{}] {}", time_str, msg);
            
            // Color code different types of messages
            if msg.contains("ERROR") || msg.contains("Failed") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Red)))
            } else if msg.contains("WARNING") || msg.contains("WARN") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Yellow)))
            } else if msg.contains("SUCCESS") || msg.contains("created successfully") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Green)))
            } else if msg.contains("Response status: 2") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Green)))
            } else if msg.contains("Response status:") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Yellow)))
            } else if msg.contains("DEBUG:") {
                Line::from(Span::styled(formatted_msg, Style::default().fg(Color::Gray)))
            } else {
                Line::from(formatted_msg)
            }
        })
        .collect();

    let debug_paragraph = Paragraph::new(debug_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((0, 0));

    f.render_widget(debug_paragraph, area);
}
