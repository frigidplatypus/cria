fn check_viewport_size(area: Rect, min_width: u16, min_height: u16, title: &str, f: &mut Frame) -> bool {
    if area.width < min_width || area.height < min_height {
        let error_block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let error_paragraph = Paragraph::new("Viewport too small to display modal. Please resize your terminal.")
            .block(error_block)
            .alignment(Alignment::Center);
        f.render_widget(Clear, area);
        f.render_widget(error_paragraph, area);
        return true;
    }
    false
}
use crate::tui::app::state::App;
use crate::tui::app::suggestion_mode::SuggestionMode;
use crate::tui::utils::{get_label_color, get_project_color};
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear, Wrap, List, ListItem};
use ratatui::text::{Line, Span};

fn colorize_quickadd_input<'a>(input: &'a str, app: &'a App) -> Vec<ratatui::text::Span<'a>> {
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
            
            // Check if next character is '[' for bracketed syntax
            if let Some(&(_, '[')) = chars.peek() {
                // Skip the '['
                chars.next();
                end += 1;
                
                // Find the closing ']'
                while let Some(&(j, nc)) = chars.peek() {
                    chars.next();
                    end = j + nc.len_utf8();
                    if nc == ']' {
                        break;
                    }
                }
            } else {
                // Find end of token (space or end of string)
                while let Some(&(j, nc)) = chars.peek() {
                    if nc == ' ' || nc == '\n' {
                        break;
                    }
                    chars.next();
                    end = j + nc.len_utf8();
                }
            }
            
            let token = &input[start..end];
            if c == '*' {
                // Label
                let label_name = token.trim_start_matches('*').trim_matches(['[', ']'].as_ref());
                let color = get_label_color(label_name, app);
                spans.push(ratatui::text::Span::styled(token, Style::default().fg(color)));
            } else if c == '+' {
                // Project
                let project_name = token.trim_start_matches('+').trim_matches(['[', ']'].as_ref());
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
    let modal_height = 22;
    if check_viewport_size(area, 40, modal_height, " Quick Add ", f) { return; }
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
                    Some(SuggestionMode::Label) => (get_label_color(s, app), "*"),
                    Some(SuggestionMode::Project) => (get_project_color(s, app), "+"),
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
        Line::from(vec![Span::raw("• "), Span::styled("new-label:urgent Buy groceries", Style::default().fg(Color::White)), Span::raw(" - creates & applies new label")]),
        Line::from(vec![Span::raw("• "), Span::styled("new-project:[Work Stuff] Plan meeting", Style::default().fg(Color::White)), Span::raw(" - creates & assigns new project")]),
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
    let modal_height = 22;
    if check_viewport_size(area, 40, modal_height, " Edit Task ", f) { return; }
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
                    Some(SuggestionMode::Label) => (get_label_color(s, app), "*"),
                    Some(SuggestionMode::Project) => (get_project_color(s, app), "+"),
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
        Line::from(vec![Span::raw("• "), Span::styled("new-label:urgent Buy groceries", Style::default().fg(Color::White)), Span::raw(" - creates & applies new label")]),
        Line::from(vec![Span::raw("• "), Span::styled("new-project:[Work Stuff] Plan meeting", Style::default().fg(Color::White)), Span::raw(" - creates & assigns new project")]),
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
    if check_viewport_size(area, 30, modal_height, " Confirm Action ", f) { return; }
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
    if check_viewport_size(area, 40, modal_height, " Help / Keybinds ", f) { return; }
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
        Line::from(vec![Span::styled("e", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Edit task (magic syntax)")]),
        Line::from(vec![Span::styled("E", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Edit task (form mode)")]),
        Line::from(vec![Span::styled("f", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Show filter picker")]),
        Line::from(vec![Span::styled("p", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Project picker")]),
        Line::from(vec![Span::styled("R", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Task relations")]),
        Line::from(vec![Span::styled("r", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Refresh tasks/projects/filters")]),
        Line::from(vec![Span::styled("s", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Star/unstar task")]),
        Line::from(vec![Span::styled("i", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Toggle info pane")]),
        Line::from(vec![Span::styled("x", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Toggle debug pane")]),
        Line::from(vec![Span::styled("o", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Open URL(s) found in task description/comments")]),
        Line::from(vec![Span::styled("h / l", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Switch layouts backward/forward")]),
        Line::from(vec![Span::styled("H / L", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Cycle task filters (active/all/etc)")]),
        Line::from(vec![Span::styled("S", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Make current task a subtask of another")]),
        Line::from(vec![Span::styled("B", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Bulk make tasks subtasks of current (multi-select)")]),
        Line::from(vec![Span::styled("Space", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Quick actions modal")]),
        Line::from(vec![Span::styled("S", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Subtask management (make task a subtask)")]),
        Line::from(vec![Span::styled("Ctrl+Z", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Undo last action")]),
        Line::from(vec![Span::styled("Ctrl+Y", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Redo last undone action")]),
        Line::from(vec![Span::styled(".", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Advanced features mode")]),
        Line::from(vec![Span::styled(".?", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Advanced features help")]),
        Line::raw("")
        ,Line::raw("")
        ,Line::from(vec![Span::raw(format!("cria version {}", env!("CARGO_PKG_VERSION")))])
    ];

    // Quick actions section
    if let Some(ref quick_actions) = app.config.quick_actions {
        if !quick_actions.is_empty() {
            help_lines.push(Line::raw("─ Quick Actions (Space for modal, . then key for direct) ─"));
            for action in quick_actions {
                help_lines.push(Line::from(vec![
                    Span::styled(format!("{}", action.key), Style::default().add_modifier(Modifier::BOLD)), 
                    Span::raw(format!(": {}", action.get_description()))
                ]));
            }
            help_lines.push(Line::raw(""));
        }
    }

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

pub fn draw_advanced_help_modal(f: &mut Frame, _app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.7) as u16;
    let modal_height = 25; // Increased from 20 to accommodate new content
    if check_viewport_size(area, 40, modal_height, " Advanced Features ", f) { return; }
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    f.render_widget(Clear, modal_area);
    let block = Block::default()
        .title(" Advanced Features (. key) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let help_lines = vec![
        Line::from(vec![Span::styled("Press . then a key for advanced features:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::raw(""),
        Line::from(vec![Span::styled(".a", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Attachment management (upload/view/download files)")]),
        Line::from(vec![Span::styled(".c", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Comments (coming soon)")]),
        Line::from(vec![Span::styled(".r", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Task relations (coming soon)")]),
        Line::from(vec![Span::styled(".h", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Task history (coming soon)")]),
        Line::from(vec![Span::styled(".s", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Subtasks (manage task hierarchy)")]),
        Line::from(vec![Span::styled(".t", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Time tracking (coming soon)")]),
        Line::from(vec![Span::styled(".?", Style::default().add_modifier(Modifier::BOLD)), Span::raw(": Show this help")]),
        Line::raw(""),
        Line::from(vec![Span::styled("URL Modal Navigation:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("When viewing URLs found in tasks (press 'o'):")]),
        Line::from(vec![Span::styled("  ↑/k, ↓/j", Style::default().fg(Color::Yellow)), Span::raw(": Navigate URL list")]),
        Line::from(vec![Span::styled("  Enter", Style::default().fg(Color::Green)), Span::raw(": Open selected URL in browser")]),
        Line::from(vec![Span::styled("  Esc", Style::default().fg(Color::Red)), Span::raw(": Cancel and close modal")]),
        Line::raw(""),
        Line::from(vec![Span::styled("Subtask Management Keybindings:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("Direct keybindings for subtask operations:")]),
        Line::from(vec![Span::styled("  S", Style::default().fg(Color::Yellow)), Span::raw(": Make current task a subtask of another")]),
        Line::from(vec![Span::styled("  B", Style::default().fg(Color::Yellow)), Span::raw(": Bulk make multiple tasks subtasks of current")]),
        Line::from(vec![Span::styled("  .s", Style::default().fg(Color::Yellow)), Span::raw(": Access subtask menu (add subtasks to current)")]),
        Line::raw(""),
        Line::from(vec![Span::styled("Note:", Style::default().fg(Color::Yellow)), Span::raw(" Attachment management and subtasks are fully functional.")]),
        Line::raw(""),
        Line::from(vec![Span::styled("Press q or ESC to close", Style::default().fg(Color::Gray))]),
    ];
    let help_paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    f.render_widget(help_paragraph, modal_area);
}

pub fn draw_advanced_features_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    if check_viewport_size(area, 40, 12, " Advanced Features ", f) { return; }
    
    // Define advanced features
    let advanced_features = vec![
        ("a", "Attachment Management", "View and manage task attachments", true),
        ("c", "Comments", "View and add task comments", false),
        ("r", "Task Relations", "Manage task dependencies and links", false),
        ("h", "Task History", "View task modification history", false),
        ("s", "Subtasks", "Manage subtasks and task hierarchy", true),
        ("t", "Time Tracking", "Track time spent on tasks", false),
    ];
    
    // Calculate modal size
    let num_features = advanced_features.len();
    let base_height = 5; // borders, title, instructions
    let feature_height = num_features as u16;
    let modal_height = (base_height + feature_height).min(area.height - 4);
    let modal_width = 70.min(area.width - 4);
    
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);
    
    let mut lines = vec![];
    
    lines.push(Line::from(vec![
        Span::raw("Select a feature (Enter to activate, Esc/q to cancel):")
    ]));
    lines.push(Line::raw(""));
    
    for (i, (key, title, description, available)) in advanced_features.iter().enumerate() {
        let is_selected = i == app.selected_advanced_feature_index;
        
        let key_style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Magenta).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
        };
        
        let title_style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        let desc_style = if is_selected {
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        
        let status_style = if *available {
            if is_selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            }
        } else {
            if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            }
        };
        
        let mut feature_spans = vec![
            Span::styled(format!(" {} ", key), key_style),
            Span::raw(" "),
            Span::styled(*title, title_style),
            Span::raw(" - "),
            Span::styled(*description, desc_style),
            Span::raw(" "),
        ];
        
        if *available {
            feature_spans.push(Span::styled("(Available)", status_style));
        } else {
            feature_spans.push(Span::styled("(Coming Soon)", status_style));
        }
        
        lines.push(Line::from(feature_spans));
    }
    
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("Navigation: ", Style::default().fg(Color::Cyan)),
        Span::raw("↑/↓ to select, Enter to activate, Esc/q to cancel")
    ]));
    
    let block = Block::default()
        .title(" Advanced Features ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    
    f.render_widget(para, modal_area);
}

pub fn draw_sort_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.5) as u16;
    let modal_height = (app.sort_options.len() as u16 + 4).min(area.height - 4);
    if check_viewport_size(area, 30, 10, " Sort Tasks ", f) { return; }
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

pub fn draw_quick_actions_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    if check_viewport_size(area, 30, 10, " Quick Actions ", f) { return; }
    
    // Calculate modal size based on number of quick actions
    let quick_actions = app.config.quick_actions.as_ref();
    let num_actions = quick_actions.map(|qa| qa.len()).unwrap_or(0);
    
    // Base height for borders, title, and instructions
    let base_height = 5;
    let action_height = num_actions as u16;
    let modal_height = (base_height + action_height).min(area.height - 4);
    let modal_width = 60.min(area.width - 4);
    
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);
    
    let mut lines = vec![];
    
    if let Some(ref quick_actions) = quick_actions {
        if quick_actions.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("No quick actions configured.", Style::default().fg(Color::Gray))
            ]));
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::raw("Add quick actions to your "),
                Span::styled("config.yaml", Style::default().fg(Color::Yellow)),
                Span::raw(" file.")
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Available Quick Actions:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            ]));
            lines.push(Line::raw(""));
            
            for (i, action) in quick_actions.iter().enumerate() {
                let key_style = if i == app.selected_quick_action_index {
                    Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                };
                
                let is_selected = i == app.selected_quick_action_index;
                
                // Create colorized description spans
                let mut description_spans = vec![
                    Span::styled(format!(" {} ", action.key), key_style),
                    Span::raw(" "),
                ];
                
                // Add colorized description based on action type
                match action.action.as_str() {
                    "project" => {
                        let base_style = if is_selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        
                        description_spans.push(Span::styled("Move to project: ", base_style.fg(Color::White)));
                        
                        let project_color = get_project_color(&action.target, app);
                        let project_style = if is_selected {
                            base_style.fg(project_color).add_modifier(Modifier::BOLD)
                        } else {
                            base_style.fg(project_color)
                        };
                        description_spans.push(Span::styled(&action.target, project_style));
                    },
                    "label" => {
                        let base_style = if is_selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        
                        description_spans.push(Span::styled("Add label: ", base_style.fg(Color::White)));
                        
                        let label_color = get_label_color(&action.target, app);
                        let label_style = if is_selected {
                            base_style.fg(label_color).add_modifier(Modifier::BOLD)
                        } else {
                            base_style.fg(label_color)
                        };
                        description_spans.push(Span::styled(&action.target, label_style));
                    },
                    "priority" => {
                        let base_style = if is_selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        
                        description_spans.push(Span::styled("Set priority to: ", base_style.fg(Color::White)));
                        
                        // Color priority based on level (1=low, 5=high)
                        let priority_color = match action.target.as_str() {
                            "1" => Color::Green,   // Low priority
                            "2" => Color::Yellow,  // Medium-low priority  
                            "3" => Color::LightBlue, // Medium priority
                            "4" => Color::Magenta, // High priority
                            "5" => Color::Red,     // Very high priority
                            _ => Color::White,     // Unknown priority
                        };
                        
                        let priority_style = if is_selected {
                            base_style.fg(priority_color).add_modifier(Modifier::BOLD)
                        } else {
                            base_style.fg(priority_color)
                        };
                        description_spans.push(Span::styled(&action.target, priority_style));
                    },
                    _ => {
                        // Fallback for unknown action types
                        let desc_style = if is_selected {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        description_spans.push(Span::styled(action.get_description(), desc_style));
                    }
                }
                
                lines.push(Line::from(description_spans));
            }
        }
    } else {
        lines.push(Line::from(vec![
            Span::styled("No quick actions configured.", Style::default().fg(Color::Gray))
        ]));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("Add quick actions to your "),
            Span::styled("config.yaml", Style::default().fg(Color::Yellow)),
            Span::raw(" file.")
        ]));
    }
    
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate • "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Select • "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel")
    ]));
    
    let block = Block::default()
        .title(" Quick Actions ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    
    f.render_widget(paragraph, modal_area);
}

pub fn draw_subtask_picker_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.7) as u16;
    let modal_height = 20;
    if check_viewport_size(area, 40, modal_height, " Subtask Management ", f) { return; }
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    f.render_widget(Clear, modal_area);
    
    // Split modal into sections
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Min(12),   // Task list
            Constraint::Length(3), // Help text
        ])
        .split(modal_area);
    
    // Determine title and instructions based on operation
    let (title, instructions, supports_multi_select) = match &app.subtask_operation {
        Some(crate::tui::app::state::SubtaskOperation::MakeSubtask) => {
            ("Make Subtask", "Select a parent task for the current task:", false)
        }
        Some(crate::tui::app::state::SubtaskOperation::AddSubtask) => {
            ("Add Subtask", "Select a task to make a subtask of the current task:", false)
        }
        None => ("Subtask Management", "", false),
    };
    
    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{} - Search Tasks", title))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    
    let input_paragraph = Paragraph::new(app.subtask_picker_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(input_paragraph, modal_chunks[0]);
    
    // Task list
    let list_block = Block::default()
        .borders(Borders::ALL)
        .title("Available Tasks")
        .style(Style::default().fg(Color::Blue));
    
    let list_items: Vec<ListItem> = app.filtered_subtask_tasks
        .iter()
        .enumerate()
        .map(|(i, (task_id, title))| {
            let is_highlighted = i == app.selected_subtask_picker_index;
            let is_selected = app.is_subtask_task_selected(*task_id);
            
            let style = if is_highlighted {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            // Truncate title if too long
            let display_width = modal_chunks[1].width.saturating_sub(12) as usize; // Account for borders, ID, and selection indicator
            let truncated_title = if title.len() > display_width {
                format!("{}...", &title[..display_width.saturating_sub(3)])
            } else {
                title.clone()
            };
            
            // Create selection indicator
            let selection_indicator = if supports_multi_select {
                if is_selected { "[✓] " } else { "[ ] " }
            } else {
                ""
            };
            
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(selection_indicator, Style::default().fg(Color::Green)),
                    Span::styled(format!("#{} ", task_id), Style::default().fg(Color::Gray)),
                    Span::styled(truncated_title, Style::default().fg(Color::White)),
                ]),
            ]).style(style)
        })
        .collect();
    
    let list = List::new(list_items)
        .block(list_block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));
    
    f.render_widget(list, modal_chunks[1]);
    
    // Help text
    let mut help_text = vec![
        Line::from(vec![
            Span::styled(instructions, Style::default().fg(Color::Cyan))
        ]),
    ];
    
    if supports_multi_select {
        help_text.push(Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(" Toggle • "),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Apply • "),
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate • "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel")
        ]));
    } else {
        help_text.push(Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Select • "),
            Span::styled("↑/↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate • "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel")
        ]));
    }
    
    let help_block = Block::default()
        .borders(Borders::ALL)
        .title("Instructions")
        .style(Style::default().fg(Color::Gray));
    
    let help_paragraph = Paragraph::new(help_text)
        .block(help_block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(help_paragraph, modal_chunks[2]);
}

// Relations modals - DISABLED: Incomplete feature
/*
pub fn draw_relations_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = 20;
    if check_viewport_size(area, 40, modal_height, " Task Relations ", f) { return; }
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    f.render_widget(Clear, modal_area);
    
    let block = Block::default()
        .title(" Task Relations ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let mut lines = vec![];
    
    if let Some(task_id) = app.relations_task_id {
        if let Some(task) = app.all_tasks.iter().find(|t| t.id == task_id) {
            lines.push(Line::from(vec![
                Span::styled("Task: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(&task.title, Style::default().fg(Color::White))
            ]));
            lines.push(Line::raw(""));
            
            if let Some(related_tasks) = &task.related_tasks {
                if related_tasks.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("No relations found.", Style::default().fg(Color::Gray))
                    ]));
                } else {
                    for (relation_type, tasks) in related_tasks {
                        if !tasks.is_empty() {
                            let relation_kind = match relation_type.as_str() {
                                "blocking" => "🚫 Blocking",
                                "blocked" => "⛔ Blocked by",
                                "subtask" => "📋 Subtask of",
                                "parenttask" => "📁 Parent of",
                                "related" => "🔗 Related to",
                                "precedes" => "⏭️ Precedes",
                                "follows" => "⏮️ Follows",
                                "duplicateof" => "📄 Duplicate of",
                                "duplicates" => "📄 Duplicates",
                                _ => relation_type,
                            };
                            
                            lines.push(Line::from(vec![
                                Span::styled(format!("{}: ", relation_kind), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                            ]));
                            
                            for related_task in tasks {
                                let status_indicator = if related_task.done { "✓" } else { "○" };
                                let task_style = if related_task.done {
                                    Style::default().fg(Color::Green)
                                } else if relation_type == "blocked" && !related_task.done {
                                    Style::default().fg(Color::Red) // Highlight blocking tasks
                                } else {
                                    Style::default().fg(Color::White)
                                };
                                
                                lines.push(Line::from(vec![
                                    Span::raw("  "),
                                    Span::styled(status_indicator, task_style),
                                    Span::raw(" "),
                                    Span::styled(format!("#{} ", related_task.id), Style::default().fg(Color::Gray)),
                                    Span::styled(&related_task.title, task_style)
                                ]));
                            }
                            lines.push(Line::raw(""));
                        }
                    }
                }
            } else {
                lines.push(Line::from(vec![
                    Span::styled("No relations found.", Style::default().fg(Color::Gray))
                ]));
            }
        }
    }
    
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("a", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Add relation • "),
        Span::styled("d", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Delete • "),
        Span::styled("r", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" Refresh • "),
        Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Close")
    ]));
    
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Left);
    
    f.render_widget(paragraph, modal_area);
}
*/

/*
pub fn draw_add_relation_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = 15;
    if check_viewport_size(area, 30, modal_height, " Add Relation ", f) { return; }
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    f.render_widget(Clear, modal_area);
    
    // Split modal into sections
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input field
            Constraint::Length(3), // Relation type selector
            Constraint::Min(6),    // Help text
        ])
        .split(modal_area);
    
    // Input field
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title("Task ID or Title")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green));
    
    let input_paragraph = Paragraph::new(app.add_relation_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(input_paragraph, modal_chunks[0]);
    
    // Set cursor position
    let cursor_x = modal_chunks[0].x + 1 + app.add_relation_cursor_position as u16;
    let cursor_y = modal_chunks[0].y + 1;
    if cursor_x < modal_chunks[0].x + modal_chunks[0].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    
    // Relation type selector
    let relation_text = if let Some(kind) = app.get_selected_relation_kind() {
        format!("Relation: {}", kind.display_name())
    } else {
        "Relation: None selected".to_string()
    };
    
    let relation_block = Block::default()
        .borders(Borders::ALL)
        .title("Relation Type (↑/↓ to change)")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Blue));
    
    let relation_paragraph = Paragraph::new(relation_text)
        .block(relation_block)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    
    f.render_widget(relation_paragraph, modal_chunks[1]);
    
    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("Add Task Relation", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]),
        Line::raw(""),
        Line::from("Enter a task ID (e.g., 123) or task title to search"),
        Line::from("Use ↑/↓ or Tab to change relation type"),
        Line::raw(""),
        Line::from("Available relation types:"),
        Line::from("• Blocking/Blocked - Task dependencies"),
        Line::from("• Subtask/Parent - Hierarchical relationships"),
        Line::from("• Related - General associations"),
        Line::from("• Precedes/Follows - Sequential ordering"),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Create • "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel")
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
*/

pub fn draw_add_subtask_modal(f: &mut Frame, app: &App) {
    let area = f.size();
    let modal_width = (area.width as f32 * 0.6) as u16;
    let modal_height = 10;
    if check_viewport_size(area, 40, modal_height, " Add Subtask ", f) { return; }
    
    let x = (area.width.saturating_sub(modal_width)) / 2;
    let y = (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect { x, y, width: modal_width, height: modal_height };
    
    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);
    
    // Layout: parent task info (2), input (3), help (rest)
    let modal_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Parent task info
            Constraint::Length(3), // Input field
            Constraint::Min(3),    // Help text
        ])
        .split(modal_area);
    
    // Show parent task info
    if let Some(parent_task) = app.get_selected_task() {
        let parent_info = Line::from(vec![
            Span::styled("Adding subtask to: ", Style::default().fg(Color::Gray)),
            Span::styled(&parent_task.title, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        ]);
        
        let parent_block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .style(Style::default().fg(Color::Cyan));
        
        let parent_paragraph = Paragraph::new(vec![parent_info])
            .block(parent_block)
            .alignment(Alignment::Center);
        
        f.render_widget(parent_paragraph, modal_chunks[0]);
    }
    
    // Input field
    let input_text = app.get_add_subtask_input();
    let input_spans = vec![Span::raw(input_text)];
    
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(" Subtask Title ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    
    let input_paragraph = Paragraph::new(vec![Line::from(input_spans)])
        .block(input_block)
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(input_paragraph, modal_chunks[1]);
    
    // Set cursor position
    let cursor_x = modal_chunks[1].x + 1 + app.add_subtask_cursor_position as u16;
    let cursor_y = modal_chunks[1].y + 1;
    if cursor_x < modal_chunks[1].x + modal_chunks[1].width - 1 {
        f.set_cursor(cursor_x, cursor_y);
    }
    
    // Help text
    let help_lines = vec![
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Create subtask • "),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel")
        ])
    ];
    
    let help_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .style(Style::default().fg(Color::Cyan));
    
    let help_paragraph = Paragraph::new(help_lines)
        .block(help_block)
        .alignment(Alignment::Center);
    
    f.render_widget(help_paragraph, modal_chunks[2]);
}