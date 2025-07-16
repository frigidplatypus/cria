// Mock terminal testing for modal rendering
// This tests the actual UI drawing functions

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::state::App;
use cria::tui::ui::modals::{draw_quick_actions_modal, draw_help_modal, draw_sort_modal};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 30);  // Increased height to accommodate modals
    Terminal::new(backend).unwrap()
}

#[allow(dead_code)]
fn sample_task(id: i64, done: bool) -> cria::vikunja::models::Task {
    use chrono::{NaiveDate, TimeZone, Utc};
    cria::vikunja::models::Task {
        id,
        title: format!("Task {}", id),
        done,
        is_favorite: false,
        labels: None,
        assignees: None,
        project_id: 1,
        priority: Some(1),
        due_date: Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(2025, 6, 30).unwrap().and_hms_opt(0,0,0).unwrap())),
        start_date: None,
        description: None,
        done_at: None,
        end_date: None,
        created: None,
        updated: None,
        created_by: None,
        percent_done: None,
        position: None,
        index: None,
        identifier: None,
        hex_color: None,
        cover_image_attachment_id: None,
        bucket_id: None,
        buckets: None,
        attachments: None,
        comments: None,
        reactions: None,
        related_tasks: None,
        reminders: None,
        repeat_after: None,
        repeat_mode: None,
        subscription: None,
    }
}

#[test]
fn test_quick_actions_modal_rendering() {
    let mut terminal = create_test_terminal();
    
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.show_quick_actions_modal();
    
    // Test that rendering doesn't panic
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
    
    // Test with empty quick actions
    let mut empty_app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    empty_app.show_quick_actions_modal();
    
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &empty_app);
    }).unwrap();
}

#[test]
fn test_help_modal_rendering() {
    let mut terminal = create_test_terminal();
    
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.show_help_modal();
    
    // Test that help modal renders without panicking
    terminal.draw(|f| {
        draw_help_modal(f, &app);
    }).unwrap();
    
    // Test with no quick actions
    let mut empty_app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    empty_app.show_help_modal();
    
    terminal.draw(|f| {
        draw_help_modal(f, &empty_app);
    }).unwrap();
}

#[test]
fn test_sort_modal_rendering() {
    let mut terminal = create_test_terminal();
    
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.show_sort_modal();
    
    // Test different selected indices
    for i in 0..app.sort_options.len() {
        app.selected_sort_index = i;
        terminal.draw(|f| {
            draw_sort_modal(f, &app);
        }).unwrap();
    }
}

#[test]
fn test_modal_rendering_with_different_terminal_sizes() {
    // Test small terminal
    let mut small_terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
    
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "p".to_string(),
            action: "project".to_string(),
            target: "Personal".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.show_quick_actions_modal();
    
    small_terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
    
    // Test large terminal
    let mut large_terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    
    large_terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
}

#[test]
fn test_modal_content_validation() {
    let mut terminal = create_test_terminal();
    
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
        QuickAction {
            key: "i".to_string(),
            action: "label".to_string(),
            target: "Important".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.show_quick_actions_modal();
    
    // Test rendering and capture the output
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
    
    let buffer = terminal.backend().buffer();
    let content = buffer.content();
    
    // Check that the modal contains expected text
    let buffer_text: String = content.iter().map(|cell| cell.symbol().chars().next().unwrap_or(' ')).collect();
    
    // Basic checks - these may need adjustment based on exact rendering
    assert!(buffer_text.contains("Quick Actions") || buffer_text.contains("Available"));
    
    // Test that different selections change the display
    app.selected_quick_action_index = 1;
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
    
    app.selected_quick_action_index = 2;
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
}
