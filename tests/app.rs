// Tests for App struct and logic in src/tui/app.rs
// Place this in tests/app.rs or similar

use cria::tui::app::{App, SortOrder};
use cria::vikunja::models::{Task, Label};
use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashMap;

fn sample_task(id: i64, done: bool) -> Task {
    Task {
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
fn test_app_initialization() {
    let app = App::new_with_default_project("Inbox".to_string());
    assert!(app.running);
    assert_eq!(app.tasks.len(), 0);
    assert_eq!(app.selected_task_index, 0);
    assert_eq!(app.default_project_name, "Inbox");
}

#[test]
fn test_quick_add_modal() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    app.add_char_to_quick_add('a');
    assert_eq!(app.quick_add_input, "a");
    app.delete_char_from_quick_add();
    assert_eq!(app.quick_add_input, "");
    app.hide_quick_add_modal();
    assert!(!app.show_quick_add_modal);
}

#[test]
fn test_edit_modal() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    assert!(app.editing_task_id.is_some());
    app.add_char_to_edit('x');
    assert!(app.edit_input.ends_with('x'));
    app.delete_char_from_edit();
    app.hide_edit_modal();
    assert!(!app.show_edit_modal);
}

#[test]
fn test_task_navigation() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    for i in 0..3 { app.tasks.push(sample_task(i, false)); }
    app.next_task();
    assert_eq!(app.selected_task_index, 1);
    app.previous_task();
    assert_eq!(app.selected_task_index, 0);
    app.jump_to_bottom();
    assert_eq!(app.selected_task_index, 2);
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_task_completion_and_undo() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    let id = app.toggle_task_completion().unwrap();
    assert!(app.tasks[0].done);
    app.undo_last_action();
    assert!(!app.tasks[0].done);
}

#[test]
fn test_task_deletion_and_undo() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.request_delete_task();
    app.confirm_action();
    assert!(app.tasks.is_empty());
    app.undo_last_action();
    assert_eq!(app.tasks.len(), 1);
}

#[test]
fn test_task_filtering() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.all_tasks = vec![sample_task(1, false), sample_task(2, true)];
    app.apply_task_filter();
    assert_eq!(app.tasks.len(), 1);
    app.cycle_task_filter();
    assert_eq!(app.tasks.len(), 2);
    app.cycle_task_filter();
    assert_eq!(app.tasks.len(), 1);
}

#[test]
fn test_project_picker() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.show_project_picker();
    app.add_char_to_project_picker('W');
    assert!(app.filtered_projects.iter().any(|(_, n)| n == "Work"));
    app.select_project_picker();
    assert!(app.current_project_id.is_some());
    app.hide_project_picker();
    assert!(!app.show_project_picker);
}

#[test]
fn test_sorting() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.tasks = vec![sample_task(2, false), sample_task(1, false)];
    app.apply_sort(SortOrder::TitleAZ);
    assert!(app.tasks[0].title < app.tasks[1].title);
    app.apply_sort(SortOrder::TitleZA);
    assert!(app.tasks[0].title > app.tasks[1].title);
}

#[test]
fn test_suggestions() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.label_map.insert(1, "Urgent".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.update_suggestions("*Ur", 3);
    assert!(app.suggestions.contains(&"Urgent".to_string()));
    app.update_suggestions("+Wo", 3);
    assert!(app.suggestions.contains(&"Work".to_string()));
    app.update_suggestions("Task", 4);
    assert!(app.suggestions.is_empty());
}

#[test]
fn test_add_task() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    let task = sample_task(42, false);
    app.tasks.push(task.clone());
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].id, 42);
    assert_eq!(app.tasks[0].title, "Task 42");
}

#[test]
fn test_edit_task_title() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    let mut task = sample_task(1, false);
    app.tasks.push(task.clone());
    // Simulate editing the task title
    app.tasks[0].title = "Edited Task".to_string();
    assert_eq!(app.tasks[0].title, "Edited Task");
}

#[test]
fn test_edit_task_priority_and_labels() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    let mut task = sample_task(1, false);
    app.tasks.push(task.clone());
    // Edit priority
    app.tasks[0].priority = Some(5);
    assert_eq!(app.tasks[0].priority, Some(5));
    // Edit labels
    app.tasks[0].labels = Some(vec![Label {
        id: 99,
        title: "Important".to_string(),
        hex_color: Some("#ff0000".to_string()),
        description: None,
        created: None,
        updated: None,
        created_by: None,
    }]);
    assert_eq!(app.tasks[0].labels.as_ref().unwrap()[0].title, "Important");
}
