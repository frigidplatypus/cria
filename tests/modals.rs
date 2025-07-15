// Tests for modal event handling: quick add and edit modals

use cria::config::CriaConfig;
use cria::tui::app::state::App;

// Helper to create a sample task for modal tests
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
fn test_quick_add_modal_events() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    // Open modal
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    // Input
    app.add_char_to_quick_add('a');
    app.add_char_to_quick_add('b');
    assert_eq!(app.quick_add_input, "ab");
    // Cursor movement
    app.move_cursor_left();
    assert_eq!(app.quick_add_cursor_position, 1);
    app.move_cursor_right();
    assert_eq!(app.quick_add_cursor_position, 2);
    // Delete
    app.delete_char_from_quick_add();
    assert_eq!(app.quick_add_input, "a");
    // Close modal
    app.hide_quick_add_modal();
    assert!(!app.show_quick_add_modal);
}

#[test]
fn test_edit_modal_events() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    // Open modal
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    // Input
    app.add_char_to_edit('x');
    assert!(app.edit_input.ends_with('x'));
    // Cursor movement
    app.move_edit_cursor_left();
    app.move_edit_cursor_right();
    // Delete
    app.delete_char_from_edit();
    // Close modal
    app.hide_edit_modal();
    assert!(!app.show_edit_modal);
}

#[test]
fn test_switching_between_modals_resets_state() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    // Open quick add modal and type
    app.show_quick_add_modal();
    app.add_char_to_quick_add('x');
    assert!(app.show_quick_add_modal);
    assert_eq!(app.quick_add_input, "x");
    // Now open edit modal (should reset quick add state)
    app.tasks.push(sample_task(1, false));
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    assert!(!app.show_quick_add_modal); // quick add should be closed
    assert_eq!(app.quick_add_input, ""); // input should be reset
}

#[test]
fn test_undo_redo_from_within_modal() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    // Open edit modal
    app.show_edit_modal();
    // Complete the task (simulate action)
    let _id = app.toggle_task_completion().unwrap();
    assert!(app.tasks[0].done);
    // Undo while modal is open
    app.undo_last_action();
    assert!(!app.tasks[0].done);
    // Redo not implemented, but you could add a redo stack and test here
}
