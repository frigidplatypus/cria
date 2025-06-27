use crate::tui::app::App;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Cell, Row, Table, Paragraph, Wrap, Clear},
    text::{Line, Span},
    Frame,
};
use chrono::Datelike;

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
    // Draw header with current project
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("Project: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(app.get_current_project_name(), Style::default().fg(Color::Cyan)),
    ])])
    .block(Block::default().borders(Borders::ALL).title("cria"))
    .alignment(Alignment::Center);
    let header_area = Rect { x: 0, y: 0, width: f.size().width, height: 3 };
    f.render_widget(header, header_area);
    let body_area = Rect { x: 0, y: 3, width: f.size().width, height: f.size().height.saturating_sub(3) };

    let _main_layout = if app.show_debug_pane {
        // Three-pane layout: tasks | info | debug
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),  // Tasks
                Constraint::Percentage(30),  // Info
                Constraint::Percentage(30),  // Debug
            ])
            .split(body_area);
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
            .split(body_area);
        draw_tasks_table(f, app, chunks[0]);
        draw_task_details(f, app, chunks[1]);
    } else {
        // Single pane: just tasks
        draw_tasks_table(f, app, body_area);
    };

    // Draw modal on top if active
    if app.show_project_picker {
        draw_project_picker_modal(f, app);
    } else if app.show_quick_add_modal {
        draw_quick_add_modal(f, app);
    } else if app.show_edit_modal {
        draw_edit_modal(f, app);
    } else if app.show_confirmation_dialog {
        draw_confirmation_dialog(f, app);
    } else if app.show_filter_picker {
        draw_filter_picker_modal(f, app);
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

        // Create colored cells with icons
        let mut title_spans = get_task_icons(task);
        
        // Add task title with strikethrough if completed
        if task.done {
            title_spans.push(Span::styled(&task.title, Style::default().add_modifier(Modifier::CROSSED_OUT).fg(Color::DarkGray)));
        } else {
            title_spans.push(Span::raw(&task.title));
        }
        
        let title_cell = Cell::from(Line::from(title_spans));
        
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
        .block(Block::default().borders(Borders::ALL).title(format!("Tasks ({})", app.get_filter_display_name())));

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

        // Only show due date if it exists and is not the epoch start
        if let Some(due_date) = &task.due_date {
            if due_date.year() > 1900 { // Check if it's a real date
                details_lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(due_date.format("%Y-%m-%d %H:%M").to_string())
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Show priority if set
        if let Some(priority) = task.priority {
            if priority > 0 {
                let priority_color = match priority {
                    5 => Color::Red,
                    4 => Color::Rgb(255, 165, 0), // Orange
                    3 => Color::Yellow,
                    2 => Color::Blue,
                    1 => Color::Magenta,
                    _ => Color::White,
                };
                details_lines.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(" ", Style::default().fg(priority_color)), // nf-fa-flag
                    Span::raw(format!(" !{}", priority))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Show star if favorited
        if task.is_favorite {
            details_lines.push(Line::from(vec![
                Span::styled("Starred: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(" ", Style::default().fg(Color::Yellow)), // nf-fa-star
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

fn draw_edit_modal(f: &mut Frame, app: &App) {
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
    let input_text = app.get_edit_input();
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Task")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Blue));
    
    let input_paragraph = Paragraph::new(input_text)
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);

    // Position cursor
    let cursor_x = modal_chunks[0].x + 1 + app.edit_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }

    // Help text - same as Quick Add but with Edit context
    let help_text = vec![
        Line::from(vec![
            Span::styled("Edit with Quick Add Magic:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
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
            Span::raw(" to update • "),
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

fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    // Calculate centered modal area (60% width, 8 lines height)
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = 8;
    
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

    // Create the modal layout
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Message
            Constraint::Length(3),  // Buttons
        ])
        .split(modal_area);

    // Confirmation message
    let message_block = Block::default()
        .borders(Borders::ALL)
        .title("Confirmation")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    
    let message_paragraph = Paragraph::new(app.get_confirmation_message())
        .block(message_block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));
    f.render_widget(message_paragraph, modal_chunks[0]);

    // Buttons
    let buttons_text = vec![
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled("Y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to confirm, ", Style::default().fg(Color::Gray)),
            Span::styled("N", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" or ", Style::default().fg(Color::Gray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ])
    ];

    let buttons_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Gray));
    
    let buttons_paragraph = Paragraph::new(buttons_text)
        .block(buttons_block)
        .alignment(Alignment::Center);
    f.render_widget(buttons_paragraph, modal_chunks[1]);
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

fn draw_project_picker_modal(f: &mut Frame, app: &App) {
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

fn get_task_icons(task: &crate::vikunja::models::Task) -> Vec<Span> {
    let mut icons = Vec::new();
    
    // Add checkmark if task is completed
    if task.done {
        icons.push(Span::styled("\u{f00c} ", Style::default().fg(Color::Green))); // nf-fa-check
    }
    
    // Add star icon if task is starred/favorited
    if task.is_favorite {
        icons.push(Span::styled("\u{f005} ", Style::default().fg(Color::Yellow))); // nf-fa-star
    }
    
    // Add priority flag icon based on priority level
    if let Some(priority) = task.priority {
        let (icon, color) = match priority {
            5 => ("\u{f024} ", Color::Red),          // !5 - Highest priority (nf-fa-flag)
            4 => ("\u{f024} ", Color::Rgb(255, 165, 0)), // !4 - Orange flag
            3 => ("\u{f024} ", Color::Yellow),       // !3 - Yellow flag
            2 => ("\u{f024} ", Color::Blue),         // !2 - Blue flag
            1 => ("\u{f024} ", Color::Magenta),      // !1 - Purple flag
            _ => ("", Color::White),         // Invalid priority
        };
        
        if !icon.is_empty() {
            icons.push(Span::styled(icon, Style::default().fg(color)));
        }
    }
    
    icons
}
