// Main layout and entry point for TUI drawing

use crate::tui::app::state::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Clear};

use super::task_list::draw_tasks_table;
use super::task_details::draw_task_details;
use super::modals::{draw_quick_add_modal, draw_edit_modal, draw_confirmation_dialog, draw_quick_actions_modal}; 
// Relations modals - DISABLED: Incomplete feature
// use super::modals::{draw_relations_modal, draw_add_relation_modal};
use super::form_edit::draw_form_edit_modal;
use super::pickers::{draw_project_picker_modal, draw_filter_picker_modal, draw_label_picker_modal};

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
    // Use full screen area (no header)
    let body_area = f.size();

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
        // Info pane is toggled off: always draw the task list in the full area.
        // This ensures a valid view is always rendered and prevents empty or broken UI states.
        draw_tasks_table(f, app, body_area);
    };

    // Draw modal on top if active
    if app.show_help_modal {
        crate::tui::ui::modals::draw_help_modal(f, app);
    } else if app.show_advanced_help_modal {
        crate::tui::ui::modals::draw_advanced_help_modal(f, app);
    } else if app.show_advanced_features_modal {
        crate::tui::ui::modals::draw_advanced_features_modal(f, app);
    } else if app.show_sort_modal {
        crate::tui::ui::modals::draw_sort_modal(f, app);
    } else if app.show_form_edit_modal {
        draw_form_edit_modal(f, app);
        // If a picker is also open, draw it on top of the form editor
        if app.show_project_picker {
            draw_project_picker_modal(f, app);
        } else if app.show_label_picker {
            draw_label_picker_modal(f, app);
        }
    } else if app.show_project_picker {
        draw_project_picker_modal(f, app);
    } else if app.show_label_picker {
        draw_label_picker_modal(f, app);
    } else if app.show_quick_add_modal {
        draw_quick_add_modal(f, app);
    } else if app.show_edit_modal {
        draw_edit_modal(f, app);
    } else if app.show_confirmation_dialog {
        draw_confirmation_dialog(f, app);
    } else if app.show_filter_picker {
        draw_filter_picker_modal(f, app);
    // Relations modals - DISABLED: Incomplete feature
    // } else if app.show_relations_modal {
    //     if app.show_add_relation_modal {
    //         draw_add_relation_modal(f, app);
    //     } else {
    //         draw_relations_modal(f, app);
    //     }
    } else if app.show_quick_actions_modal {
        draw_quick_actions_modal(f, app);
    } else if app.show_attachment_modal {
        if let Some(ref modal) = app.attachment_modal {
            modal.draw(f, f.size());
        }
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
    
    // Draw layout notification if active
    if let Some(notification) = app.get_layout_notification() {
        let notification_width = (notification.len() as u16 + 4).min(f.size().width / 2);
        let notification_area = Rect {
            x: f.size().width.saturating_sub(notification_width + 2),
            y: f.size().height.saturating_sub(6), // Bottom right, above toast
            width: notification_width,
            height: 3,
        };
        let notification_msg = Paragraph::new(notification.clone())
            .block(Block::default().borders(Borders::ALL).title("Layout"))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, notification_area);
        f.render_widget(notification_msg, notification_area);
    }

    // Draw toast notification if active
    if let Some(toast) = app.get_toast() {
        let toast_width = (toast.len() as u16 + 4).min(f.size().width / 2);
        let toast_area = Rect {
            x: f.size().width.saturating_sub(toast_width + 2),
            y: f.size().height.saturating_sub(3), // Bottom right
            width: toast_width,
            height: 3,
        };
        let toast_msg = Paragraph::new(toast.clone())
            .block(Block::default().borders(Borders::ALL).title("Success"))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        f.render_widget(Clear, toast_area);
        f.render_widget(toast_msg, toast_area);
    }
}
