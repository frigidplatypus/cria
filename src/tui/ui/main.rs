// Main layout and entry point for TUI drawing

use crate::tui::app::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear};
use ratatui::text::{Line, Span};

use super::task_list::draw_tasks_table;
use super::task_details::draw_task_details;
use super::modals::{draw_quick_add_modal, draw_edit_modal, draw_confirmation_dialog, draw_quick_actions_modal};
use super::pickers::{draw_project_picker_modal, draw_filter_picker_modal};

pub fn hex_to_color(hex: &str) -> Color {
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
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(body_area);
        draw_tasks_table(f, app, horizontal_chunks[0]);
        if app.show_info_pane {
            draw_task_details(f, app, horizontal_chunks[1]);
        }
        // draw_debug_pane(f, app, horizontal_chunks[2]);
    } else if app.show_info_pane {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(body_area);
        draw_tasks_table(f, app, chunks[0]);
        draw_task_details(f, app, chunks[1]);
    } else {
        draw_tasks_table(f, app, body_area);
    };

    // Draw modal on top if active
    if app.show_help_modal {
        crate::tui::ui::modals::draw_help_modal(f, app);
    } else if app.show_sort_modal {
        crate::tui::ui::modals::draw_sort_modal(f, app);
    } else if app.show_project_picker {
        draw_project_picker_modal(f, app);
    } else if app.show_quick_add_modal {
        draw_quick_add_modal(f, app);
    } else if app.show_edit_modal {
        draw_edit_modal(f, app);
    } else if app.show_confirmation_dialog {
        draw_confirmation_dialog(f, app);
    } else if app.show_filter_picker {
        draw_filter_picker_modal(f, app);
    } else if app.show_quick_actions_modal {
        draw_quick_actions_modal(f, app);
    }

    // Draw refreshing indicator if refreshing
    if app.refreshing {
        let refresh_area = Rect {
            x: 0,
            y: f.size().height.saturating_sub(1),
            width: f.size().width,
            height: 1,
        };
        let refresh_msg = Paragraph::new("Refreshing...")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, refresh_area);
        f.render_widget(refresh_msg, refresh_area);
    }
}
