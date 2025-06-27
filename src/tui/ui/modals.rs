// Modal rendering (quick add, edit, confirmation)

use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear, Wrap};
use ratatui::text::{Line, Span};

pub fn draw_quick_add_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 18;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field with title
            Constraint::Min(14),   // Help text
        ])
        .split(modal_area);
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
    let cursor_x = modal_chunks[0].x + 1 + app.quick_add_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    let help_text = vec![
        Line::from(vec![
            Span::styled("Quick Add Magic Examples:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![Span::raw("• "), Span::styled("Buy groceries *shopping *urgent", Style::default().fg(Color::White)), Span::raw(" - adds labels")]),
        Line::from(vec![Span::raw("• "), Span::styled("Review PR @john", Style::default().fg(Color::White)), Span::raw(" - assigns to user")]),
        Line::from(vec![Span::raw("• "), Span::styled("Fix bug +work !3", Style::default().fg(Color::White)), Span::raw(" - sets project & priority")]),
        Line::from(vec![Span::raw("• "), Span::styled("Call mom tomorrow at 2pm", Style::default().fg(Color::White)), Span::raw(" - sets due date")]),
        Line::from(vec![Span::raw("• "), Span::styled("Team meeting every Monday", Style::default().fg(Color::White)), Span::raw(" - recurring task")]),
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

pub fn draw_edit_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 18;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field with title
            Constraint::Min(14),   // Help text
        ])
        .split(modal_area);
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
    let cursor_x = modal_chunks[0].x + 1 + app.edit_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    let help_text = vec![
        Line::from(vec![
            Span::styled("Edit with Quick Add Magic:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![Span::raw("• "), Span::styled("Buy groceries *shopping *urgent", Style::default().fg(Color::White)), Span::raw(" - adds labels")]),
        Line::from(vec![Span::raw("• "), Span::styled("Review PR @john", Style::default().fg(Color::White)), Span::raw(" - assigns to user")]),
        Line::from(vec![Span::raw("• "), Span::styled("Fix bug +work !3", Style::default().fg(Color::White)), Span::raw(" - sets project & priority")]),
        Line::from(vec![Span::raw("• "), Span::styled("Call mom tomorrow at 2pm", Style::default().fg(Color::White)), Span::raw(" - sets due date")]),
        Line::from(vec![Span::raw("• "), Span::styled("Team meeting every Monday", Style::default().fg(Color::White)), Span::raw(" - recurring task")]),
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

pub fn draw_confirmation_dialog(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = 8;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let block = Block::default()
        .title(" Confirm Action ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    f.render_widget(block, modal_area);

    let buttons_text: Vec<Line> = vec![
        Line::from("Press "),
        Line::from("Y").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(" to confirm, or "),
        Line::from("N").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(" to cancel."),
    ];
    let buttons_block = Block::default().borders(Borders::NONE);
    let buttons_paragraph = Paragraph::new(buttons_text)
        .block(buttons_block)
        .alignment(Alignment::Center);
    f.render_widget(buttons_paragraph, modal_area);
}
