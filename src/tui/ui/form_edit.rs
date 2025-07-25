use crate::tui::app::state::App;
use crate::tui::app::form_edit_state::FormEditState;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear, Wrap};
use ratatui::text::{Line, Span};

pub fn draw_form_edit_modal(f: &mut Frame, app: &App) {
    if let Some(form) = &app.form_edit_state {
        let area = f.size();
        let modal_width = (area.width as f32 * 0.9) as u16;
        let modal_height = (area.height as f32 * 0.9) as u16;
        let x = (area.width.saturating_sub(modal_width)) / 2;
        let y = (area.height.saturating_sub(modal_height)) / 2;
        let modal_area = Rect { x, y, width: modal_width, height: modal_height };
        
        f.render_widget(Clear, modal_area);
        
        let block = Block::default()
            .title(" Task Editor (Form Mode) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        
        f.render_widget(block, modal_area);
        
        let inner_area = Rect {
            x: modal_area.x + 1,
            y: modal_area.y + 1,
            width: modal_area.width - 2,
            height: modal_area.height - 2,
        };
        
        // Split the inner area into main form and help section
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(15),    // Main form area
                Constraint::Length(6),  // Help section
            ])
            .split(inner_area);
        
        // Render main form
        render_form_fields(f, chunks[0], app, form);
        
        // Render help section
        render_help_section(f, chunks[1], form);
    }
}

fn render_form_fields(f: &mut Frame, area: Rect, app: &App, form: &FormEditState) {
    let mut lines = Vec::new();
    
    // Title field
    let title_style = if form.field_index == 0 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let title_prefix = if form.field_index == 0 { "► " } else { "  " };
    lines.push(Line::from(vec![
        Span::styled(title_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Title: ", title_style),
        Span::styled(&form.title, if form.field_index == 0 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
    ]));
    
    // Description field
    let desc_style = if form.field_index == 1 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let desc_prefix = if form.field_index == 1 { "► " } else { "  " };
    let desc_text = if form.description.is_empty() { 
        "<empty>" 
    } else { 
        &form.description 
    };
    lines.push(Line::from(vec![
        Span::styled(desc_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Description: ", desc_style),
        Span::styled(desc_text, if form.field_index == 1 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
    ]));
    
    // Due Date field
    let due_style = if form.field_index == 2 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let due_prefix = if form.field_index == 2 { "► " } else { "  " };
    let due_text = form.due_date.as_deref().unwrap_or("<not set>");
    lines.push(Line::from(vec![
        Span::styled(due_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Due Date: ", due_style),
        Span::styled(due_text, if form.field_index == 2 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (YYYY-MM-DD)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Start Date field
    let start_style = if form.field_index == 3 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let start_prefix = if form.field_index == 3 { "► " } else { "  " };
    let start_text = form.start_date.as_deref().unwrap_or("<not set>");
    lines.push(Line::from(vec![
        Span::styled(start_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Start Date: ", start_style),
        Span::styled(start_text, if form.field_index == 3 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (YYYY-MM-DD)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Priority field
    let prio_style = if form.field_index == 4 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let prio_prefix = if form.field_index == 4 { "► " } else { "  " };
    let prio_text = form.priority.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string());
    lines.push(Line::from(vec![
        Span::styled(prio_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Priority: ", prio_style),
        Span::styled(&prio_text, if form.field_index == 4 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (0-5, 0=none)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Project field (with name lookup)
    let proj_style = if form.field_index == 5 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let proj_prefix = if form.field_index == 5 { "► " } else { "  " };
    let project_name = app.project_map.get(&form.project_id)
        .map(|name| format!("{} (ID: {})", name, form.project_id))
        .unwrap_or_else(|| format!("Unknown Project (ID: {})", form.project_id));
    lines.push(Line::from(vec![
        Span::styled(proj_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Project: ", proj_style),
        Span::styled(&project_name, if form.field_index == 5 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (Space to pick)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Labels field (with name lookup)
    let labels_style = if form.field_index == 6 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let labels_prefix = if form.field_index == 6 { "► " } else { "  " };
    let labels_text = if form.label_ids.is_empty() {
        "<none>".to_string()
    } else {
        form.label_ids.iter()
            .filter_map(|id| app.label_map.get(id))
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    lines.push(Line::from(vec![
        Span::styled(labels_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Labels: ", labels_style),
        Span::styled(&labels_text, if form.field_index == 6 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (Space to pick)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Assignees field
    let assign_style = if form.field_index == 7 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let assign_prefix = if form.field_index == 7 { "► " } else { "  " };
    let assign_text = if form.assignee_ids.is_empty() { 
        "<none>" 
    } else { 
        &format!("{:?}", form.assignee_ids) 
    };
    lines.push(Line::from(vec![
        Span::styled(assign_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Assignees: ", assign_style),
        Span::styled(assign_text, if form.field_index == 7 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
    ]));
    
    // Is Favorite field
    let fav_style = if form.field_index == 8 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let fav_prefix = if form.field_index == 8 { "► " } else { "  " };
    let fav_text = if form.is_favorite { "★ Yes" } else { "☆ No" };
    lines.push(Line::from(vec![
        Span::styled(fav_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Favorite: ", fav_style),
        Span::styled(fav_text, if form.field_index == 8 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
        Span::styled(" (Space to toggle)", Style::default().fg(Color::DarkGray)),
    ]));
    
    // Comment field
    let comment_style = if form.field_index == 9 {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    let comment_prefix = if form.field_index == 9 { "► " } else { "  " };
    let comment_text = if form.comment.is_empty() { 
        "Type your comment, then press Enter to save" 
    } else { 
        &form.comment 
    };
    lines.push(Line::from(vec![
        Span::styled(comment_prefix, Style::default().fg(Color::Yellow)),
        Span::styled("Add Comment: ", comment_style),
        Span::styled(comment_text, if form.field_index == 9 { 
            Style::default().fg(Color::White).bg(Color::DarkGray) 
        } else { 
            Style::default().fg(Color::Gray) 
        }),
    ]));
    
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    
    f.render_widget(paragraph, area);
}

fn render_help_section(f: &mut Frame, area: Rect, form: &FormEditState) {
    let mut help_lines = Vec::new();
    
    help_lines.push(Line::from(vec![
        Span::styled("Navigation:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw("/"),
        Span::styled("Shift+Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" - Next/Previous field  "),
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" - Navigate fields"),
    ]));
    
    help_lines.push(Line::from(vec![
        Span::styled("Editing:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("     "),
        Span::styled("Type", Style::default().fg(Color::Yellow)),
        Span::raw(" - Edit text fields  "),
        Span::styled("Backspace", Style::default().fg(Color::Yellow)),
        Span::raw(" - Delete  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" - Pick/Toggle"),
    ]));
    
    help_lines.push(Line::from(vec![
        Span::styled("Actions:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("     "),
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(" - Save task  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(" - Cancel without saving"),
    ]));
    
    // Field-specific help
    match form.field_index {
        2 | 3 => {
            help_lines.push(Line::from(vec![
                Span::styled("Date Format:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("  YYYY-MM-DD (e.g., 2025-07-15) or leave empty for no date"),
            ]));
        }
        4 => {
            help_lines.push(Line::from(vec![
                Span::styled("Priority:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("     0=None, 1=Low, 2=Medium, 3=High, 4=Urgent, 5=Critical"),
            ]));
        }
        5 => {
            help_lines.push(Line::from(vec![
                Span::styled("Project:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("      Press "),
                Span::styled("Space", Style::default().fg(Color::Yellow)),
                Span::raw(" to open project picker"),
            ]));
        }
        6 => {
            help_lines.push(Line::from(vec![
                Span::styled("Labels:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("       Press "),
                Span::styled("Space", Style::default().fg(Color::Yellow)),
                Span::raw(" to open label picker"),
            ]));
        }
        8 => {
            help_lines.push(Line::from(vec![
                Span::styled("Favorite:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("     Press "),
                Span::styled("Space", Style::default().fg(Color::Yellow)),
                Span::raw(" to toggle favorite status"),
            ]));
        }
        _ => {
            help_lines.push(Line::from(vec![
                Span::styled("Tip:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("          Use Tab to navigate between fields quickly"),
            ]));
        }
    }
    
    let help_paragraph = Paragraph::new(help_lines)
        .block(Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    
    f.render_widget(help_paragraph, area);
}
