// Enhanced modal testing framework
// Tests all modal functionality without GUI

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::App;
use cria::vikunja::models::{Task, Label};
use chrono::{NaiveDate, TimeZone, Utc};

// Helper to create sample tasks and config
fn create_test_app_with_quick_actions() -> App {
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
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.project_map.insert(3, "Personal".to_string());
    
    // Add some sample tasks
    app.tasks.push(sample_task(1, false));
    app.tasks.push(sample_task(2, true));
    
    app
}

fn sample_task(id: i64, done: bool) -> Task {
    Task {
        id,
        title: format!("Task {}", id),
        done,
        is_favorite: false,
        labels: Some(vec![Label {
            id: 1,
            title: "existing".to_string(),
            hex_color: Some("#ff0000".to_string()),
            description: None,
            created: None,
            updated: None,
            created_by: None,
        }]),
        assignees: None,
        project_id: 1,
        priority: Some(1),
        due_date: Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(2025, 6, 30).unwrap().and_hms_opt(0,0,0).unwrap())),
        // ... rest of the task fields
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
fn test_quick_actions_modal_navigation() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test modal opening
    app.show_quick_actions_modal();
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
    
    // Test navigation down
    let quick_actions_count = app.config.quick_actions.as_ref().unwrap().len();
    for i in 1..quick_actions_count {
        // Simulate down arrow key
        if app.selected_quick_action_index + 1 < quick_actions_count {
            app.selected_quick_action_index += 1;
        }
        assert_eq!(app.selected_quick_action_index, i);
    }
    
    // Test navigation up
    for i in (0..quick_actions_count-1).rev() {
        if app.selected_quick_action_index > 0 {
            app.selected_quick_action_index -= 1;
        }
        assert_eq!(app.selected_quick_action_index, i);
    }
    
    // Test modal closing
    app.hide_quick_actions_modal();
    assert!(!app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
}

#[test]
fn test_quick_actions_execution() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test project change action
    let project_action = QuickAction {
        key: "w".to_string(),
        action: "project".to_string(),
        target: "Work".to_string(),
    };
    
    let original_project = app.tasks[0].project_id;
    let result = app.apply_quick_action(&project_action);
    assert!(result.is_ok());
    assert_ne!(app.tasks[0].project_id, original_project);
    assert_eq!(app.tasks[0].project_id, 2); // Work project ID
    
    // Test priority change action
    let priority_action = QuickAction {
        key: "u".to_string(),
        action: "priority".to_string(),
        target: "5".to_string(),
    };
    
    let result = app.apply_quick_action(&priority_action);
    assert!(result.is_ok());
    assert_eq!(app.tasks[0].priority, Some(5));
    
    // Test invalid priority
    let invalid_priority_action = QuickAction {
        key: "x".to_string(),
        action: "priority".to_string(),
        target: "10".to_string(),
    };
    
    let result = app.apply_quick_action(&invalid_priority_action);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Priority must be between 1 and 5"));
}

#[test]
fn test_help_modal_state() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test showing help modal
    app.show_help_modal();
    assert!(app.show_help_modal);
    
    // Test hiding help modal
    app.hide_help_modal();
    assert!(!app.show_help_modal);
    
    // Test that help modal doesn't interfere with other modals
    app.show_help_modal();
    app.show_quick_add_modal();
    assert!(!app.show_help_modal); // Should be hidden when other modal opens
    assert!(app.show_quick_add_modal);
}

#[test]
fn test_sort_modal_functionality() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test sort modal opening
    app.show_sort_modal();
    assert!(app.show_sort_modal);
    assert_eq!(app.selected_sort_index, 0);
    
    // Test sort selection navigation
    let sort_options_count = app.sort_options.len();
    for i in 1..sort_options_count {
        if app.selected_sort_index + 1 < sort_options_count {
            app.selected_sort_index += 1;
        }
        assert_eq!(app.selected_sort_index, i);
    }
    
    // Test applying sort
    app.selected_sort_index = 1; // TitleAZ
    let sort = match app.selected_sort_index {
        0 => cria::tui::app::SortOrder::Default,
        1 => cria::tui::app::SortOrder::TitleAZ,
        2 => cria::tui::app::SortOrder::TitleZA,
        3 => cria::tui::app::SortOrder::PriorityHighToLow,
        4 => cria::tui::app::SortOrder::PriorityLowToHigh,
        _ => cria::tui::app::SortOrder::Default,
    };
    app.apply_sort(sort);
    
    // Test hiding sort modal
    app.hide_sort_modal();
    assert!(!app.show_sort_modal);
}

#[test]
fn test_modal_exclusivity() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test that only one modal can be open at a time
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    
    app.show_edit_modal();
    assert!(!app.show_quick_add_modal);
    assert!(app.show_edit_modal);
    
    app.show_help_modal();
    assert!(!app.show_edit_modal);
    assert!(app.show_help_modal);
    
    app.show_quick_actions_modal();
    assert!(!app.show_help_modal);
    assert!(app.show_quick_actions_modal);
    
    app.show_sort_modal();
    assert!(!app.show_quick_actions_modal);
    assert!(app.show_sort_modal);
}

#[test]
fn test_modal_state_preservation() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test that modal state is preserved when switching
    app.show_quick_actions_modal();
    app.selected_quick_action_index = 2;
    
    // Switch to another modal and back
    app.show_help_modal();
    app.hide_help_modal();
    app.show_quick_actions_modal();
    
    // State should be reset (this is current behavior)
    assert_eq!(app.selected_quick_action_index, 0);
    
    // Test sort modal state
    app.show_sort_modal();
    app.selected_sort_index = 3;
    
    app.show_help_modal();
    app.hide_help_modal();
    app.show_sort_modal();
    
    // Sort selection should be preserved
    assert_eq!(app.selected_sort_index, 3);
}

#[test]
fn test_modal_error_handling() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test quick actions with no tasks
    app.tasks.clear();
    let action = QuickAction {
        key: "w".to_string(),
        action: "project".to_string(),
        target: "Work".to_string(),
    };
    
    let result = app.apply_quick_action(&action);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No tasks available"));
    
    // Test with invalid project name
    app.tasks.push(sample_task(1, false));
    let invalid_project_action = QuickAction {
        key: "x".to_string(),
        action: "project".to_string(),
        target: "NonexistentProject".to_string(),
    };
    
    let result = app.apply_quick_action(&invalid_project_action);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}
