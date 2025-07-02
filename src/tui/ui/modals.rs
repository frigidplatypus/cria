use crate::tui::app::App;
use crate::tui::utils::{get_label_color, get_project_color};
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear, Wrap};
use ratatui::text::{Line, Span};

fn colorize_quickadd_input<'a>(input: &'a str, app: &'a crate::tui::app::App) -> Vec<ratatui::text::Span<'a>> {
    let mut spans = Vec::new();
    let mut chars = input.char_indices().peekable();
    let mut last = 0;
    while let Some((i, c)) = chars.next() {
        if c == '*' || c == '+' {
            // Push previous text
            if last < i {
                spans.push(ratatui::text::Span::raw(&input[last..i]));
            }
            let start = i;
            let mut end = start + 1; // Start after the '*' or '+'
            // Find end of token (space or end of string)
            while let Some(&(j, nc)) = chars.peek() {
                if nc == ' ' || nc == '\n' {
                    break;
                }
                chars.next();
                end = j + nc.len_utf8();
            }
            let token = &input[start..end];
            if c == '*' {
                // Label
                let label_name = token.trim_start_matches('*');
                let color = get_label_color(label_name, app);
                spans.push(ratatui::text::Span::styled(token, Style::default().fg(color)));
            } else if c == '+' {
                // Project
                let project_name = token.trim_start_matches('+');
                let color = get_project_color(project_name, app);
                spans.push(ratatui::text::Span::styled(token, Style::default().fg(color)));
            }
            last = end;
        }
    }
    if last < input.len() {
        spans.push(ratatui::text::Span::raw(&input[last..]));
    }
    spans
}

pub fn draw_quick_add_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 22; // Increased height for more space
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    // Layout: input (3), suggestions (6), help (rest)
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field with title
            Constraint::Length(6), // Suggestions (fixed height)
            Constraint::Min(10),   // Help text
        ])
        .split(modal_area);
    let input_text = app.get_quick_add_input();
    let input_spans = colorize_quickadd_input(input_text, app);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Quick Add Task")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    let input_paragraph = Paragraph::new(vec![Line::from(input_spans)])
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);
    let cursor_x = modal_chunks[0].x + 1 + app.quick_add_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    // Render suggestions in the reserved chunk
    if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
        let max_visible = 4; // Match the visible lines in the UI
        let total = app.suggestions.len();
        let mut start = 0;
        let max_end = total.min(max_visible);
        if app.selected_suggestion >= max_end {
            start = app.selected_suggestion + 1 - max_visible;
        }
        let suggestion_lines: Vec<Line> = app.suggestions.iter().enumerate()
            .skip(start)
            .take(max_visible)
            .map(|(i, s)| {
                let (color, prefix) = match app.suggestion_mode {
                    Some(crate::tui::app::SuggestionMode::Label) => (get_label_color(s, app), "*"),
                    Some(crate::tui::app::SuggestionMode::Project) => (get_project_color(s, app), "+"),
                    _ => (Color::Gray, "")
                };
                let styled = Span::styled(format!("{}{}", prefix, s), Style::default().fg(color));
                let absolute_index = start + i;
                if absolute_index == app.selected_suggestion {
                    // Highlight with color background and black text
                    Line::from(vec![Span::styled(
                        format!("{}{}", prefix, s),
                        Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
                    )])
                } else {
                    Line::from(vec![styled])
                }
            }).collect();
        let suggestion_block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .style(Style::default().fg(Color::Gray));
        let suggestion_paragraph = Paragraph::new(suggestion_lines)
            .block(suggestion_block)
            .wrap(Wrap { trim: true });
        f.render_widget(suggestion_paragraph, modal_chunks[1]);
    } else {
        // Optionally, render an empty suggestions box for consistent UI
        let suggestion_block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .style(Style::default().fg(Color::Gray));
        let suggestion_paragraph = Paragraph::new("")
            .block(suggestion_block)
            .wrap(Wrap { trim: true });
        f.render_widget(suggestion_paragraph, modal_chunks[1]);
    }
    // Help text at the bottom
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
        Line::from("") ,
        Line::from(vec![
            Span::styled("Syntax: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("*label ", Style::default().fg(Color::Red)),
            Span::styled("@user ", Style::default().fg(Color::Blue)),
            Span::styled("+project ", Style::default().fg(Color::Magenta)),
            Span::styled("!priority ", Style::default().fg(Color::Yellow)),
            Span::styled("dates", Style::default().fg(Color::Cyan))
        ]),
        Line::from("") ,
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
    f.render_widget(help_paragraph, modal_chunks[2]);
}

pub fn draw_edit_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 22; // Match quick add modal height
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    // Layout: input (3), suggestions (6), help (rest)
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field with title
            Constraint::Length(6), // Suggestions (fixed height)
            Constraint::Min(10),   // Help text
        ])
        .split(modal_area);
    let input_text = app.get_edit_input();
    let input_spans = colorize_quickadd_input(input_text, app);
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Edit Task")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    let input_paragraph = Paragraph::new(vec![Line::from(input_spans)])
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input_paragraph, modal_chunks[0]);
    let cursor_x = modal_chunks[0].x + 1 + app.edit_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    // Render suggestions in the reserved chunk
    if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
        let max_visible = 4;
        let total = app.suggestions.len();
        let mut start = 0;
        let max_end = total.min(max_visible);
        if app.selected_suggestion >= max_end {
            start = app.selected_suggestion + 1 - max_visible;
        }
        let suggestion_lines: Vec<Line> = app.suggestions.iter().enumerate()
            .skip(start)
            .take(max_visible)
            .map(|(i, s)| {
                let (color, prefix) = match app.suggestion_mode {
                    Some(crate::tui::app::SuggestionMode::Label) => (get_label_color(s, app), "*"),
                    Some(crate::tui::app::SuggestionMode::Project) => (get_project_color(s, app), "+"),
                    _ => (Color::Gray, "")
                };
                let styled = Span::styled(format!("{}{}", prefix, s), Style::default().fg(color));
                let absolute_index = start + i;
                if absolute_index == app.selected_suggestion {
                    Line::from(vec![Span::styled(
                        format!("{}{}", prefix, s),
                        Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
                    )])
                } else {
                    Line::from(vec![styled])
                }
            }).collect();
        let suggestion_block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .style(Style::default().fg(Color::Gray));
        let suggestion_paragraph = Paragraph::new(suggestion_lines)
            .block(suggestion_block)
            .wrap(Wrap { trim: true });
        f.render_widget(suggestion_paragraph, modal_chunks[1]);
    } else {
        let suggestion_block = Block::default()
            .borders(Borders::ALL)
            .title("Suggestions")
            .style(Style::default().fg(Color::Gray));
        let suggestion_paragraph = Paragraph::new("")
            .block(suggestion_block)
            .wrap(Wrap { trim: true });
        f.render_widget(suggestion_paragraph, modal_chunks[1]);
    }
    // Help text at the bottom
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
    f.render_widget(help_paragraph, modal_chunks[2]);
}

pub fn draw_confirmation_dialog(f: &mut Frame, _app: &App) {
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

pub fn draw_help_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.7) as u16;
    let modal_height = 26;
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let block = Block::default()
        .title(" Help / Keybinds ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let mut help_lines = vec![
        Line::from(vec![Span::styled("?", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Show this help")]),
        Line::from(vec![Span::styled("q", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Quit (in main view) / Close modal (in modal)")]),
        Line::from(vec![Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Close modal/help")]),
        Line::from(vec![Span::styled("j / k", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Move selection down/up")]),
        Line::from(vec![Span::styled("Up / Down", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Move selection up/down")]),
        Line::from(vec![Span::styled("g / G", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Jump to top/bottom")]),
        Line::from(vec![Span::styled("d", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Toggle task completion")]),
        Line::from(vec![Span::styled("D", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Delete task")]),
        Line::from(vec![Span::styled("a", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Add task (quick add modal)")]),
        Line::from(vec![Span::styled("e", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Edit task")]),
        Line::from(vec![Span::styled("f", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Show filter picker")]),
        Line::from(vec![Span::styled("p", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Project picker")]),
        Line::from(vec![Span::styled("r", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Refresh tasks/projects/filters")]),
        Line::from(vec![Span::styled("s", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Star/unstar task")]),
        Line::from(vec![Span::styled("i", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Toggle info pane")]),
        Line::from(vec![Span::styled("h / l", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Cycle filters backward/forward")]),
        Line::from(vec![Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Shortcuts modal")]),
        Line::raw("")
    ];
    // Config details section
    help_lines.push(Line::raw("─ Config Details ─"));
    // Show config file path (XDG or default)
    let config_path = std::env::var("XDG_CONFIG_HOME")
        .map(|val| format!("{}/cria/config.yaml", val))
        .unwrap_or_else(|_| format!("{}/.config/cria/config.yaml", std::env::var("HOME").unwrap_or("~".to_string())));
    help_lines.push(Line::raw(format!("Config file: {}", config_path)));
    help_lines.push(Line::raw(format!("API URL: {}", app.config.api_url)));
    if let Some(ref key) = app.config.api_key {
        let key: &str = key;
        let obfuscated = if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        } else {
            "(set, hidden)".to_string()
        };
        help_lines.push(Line::raw(format!("API Key: {}", obfuscated)));
    } else if let Some(ref file) = app.config.api_key_file {
        help_lines.push(Line::raw(format!("API Key File: {}", file)));
    } else {
        help_lines.push(Line::raw("API Key: (not set)"));
    }
    if let Some(ref proj) = app.config.default_project {
        help_lines.push(Line::raw(format!("Default Project: {}", proj)));
    }
    let help_paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    f.render_widget(help_paragraph, modal_area);
}

pub fn draw_sort_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.5) as u16;
    let modal_height = (app.sort_options.len() as u16 + 4).min(area.height - 4);
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let block = Block::default()
        .title(" Sort Tasks ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let mut lines = vec![Line::raw("Select a sorting method (Enter to apply, Esc/q to cancel):")];
    for (i, opt) in app.sort_options.iter().enumerate() {
        let style = if i == app.selected_sort_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(*opt, style)));
    }
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    f.render_widget(para, modal_area);
}
