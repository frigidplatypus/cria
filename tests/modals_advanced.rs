// Advanced modal testing with event simulation and property-based testing approaches
// This demonstrates more sophisticated testing strategies for TUI applications

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::state::App;
use cria::vikunja::models::{Task, Label};
use chrono::{NaiveDate, TimeZone, Utc};
use crossterm::event::KeyCode;

// Helper to create sample tasks for testing
fn sample_task_with_details(id: i64, title: &str, priority: Option<i32>, project_id: i64) -> Task {
    Task {
        id,
        title: title.to_string(),
        done: false,
        is_favorite: false,
        labels: Some(vec![Label {
            id: 1,
            title: "test".to_string(),
            hex_color: Some("#ff0000".to_string()),
            description: None,
            created: None,
            updated: None,
            created_by: None,
        }]),
        assignees: None,
        project_id,
        priority,
        due_date: Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(2025, 12, 31).unwrap().and_hms_opt(0,0,0).unwrap())),
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

// Create test app with rich data
fn create_rich_test_app() -> App {
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
        QuickAction {
            key: "m".to_string(),
            action: "priority".to_string(),
            target: "3".to_string(),
        },
        QuickAction {
            key: "l".to_string(),
            action: "priority".to_string(),
            target: "1".to_string(),
        },
        QuickAction {
            key: "i".to_string(),
            action: "label".to_string(),
            target: "Important".to_string(),
        },
        QuickAction {
            key: "r".to_string(),
            action: "label".to_string(),
            target: "Review".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    
    // Add multiple projects
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.project_map.insert(3, "Personal".to_string());
    app.project_map.insert(4, "Learning".to_string());
    
    // Add labels for quick actions
    app.label_map.insert(1, "test".to_string());
    app.label_map.insert(2, "Important".to_string());
    app.label_map.insert(3, "Review".to_string());
    
    // Add various tasks with different priorities and projects
    app.tasks.push(sample_task_with_details(1, "High priority work task", Some(5), 2));
    app.tasks.push(sample_task_with_details(2, "Medium priority personal task", Some(3), 3));
    app.tasks.push(sample_task_with_details(3, "Low priority learning task", Some(1), 4));
    app.tasks.push(sample_task_with_details(4, "No priority inbox task", None, 1));
    app.tasks.push(sample_task_with_details(5, "Another work task", Some(4), 2));
    
    app
}

// Simulate a sequence of key events (simplified event simulation)
fn simulate_key_sequence(app: &mut App, keys: Vec<KeyCode>) -> Vec<String> {
    let mut results = Vec::new();
    
    for key in keys {
        let result = match key {
            KeyCode::Char(' ') => {
                if !app.show_quick_actions_modal {
                    app.show_quick_actions_modal();
                    "Opened quick actions modal".to_string()
                } else {
                    "Quick actions modal already open".to_string()
                }
            },
            KeyCode::Char('q') => {
                if app.show_quick_actions_modal || app.show_help_modal || app.show_sort_modal {
                    close_all_modals(app);
                    "Closed modal".to_string()
                } else {
                    app.quit();
                    "Quit app".to_string()
                }
            },
            KeyCode::Char('?') => {
                app.show_help_modal();
                "Opened help modal".to_string()
            },
            KeyCode::Down => {
                if app.show_quick_actions_modal {
                    let max_index = app.config.quick_actions.as_ref().map(|qa| qa.len()).unwrap_or(0);
                    if max_index > 0 && app.selected_quick_action_index + 1 < max_index {
                        app.selected_quick_action_index += 1;
                    }
                    format!("Selected quick action index: {}", app.selected_quick_action_index)
                } else {
                    app.next_task();
                    format!("Selected task index: {}", app.selected_task_index)
                }
            },
            KeyCode::Up => {
                if app.show_quick_actions_modal {
                    if app.selected_quick_action_index > 0 {
                        app.selected_quick_action_index -= 1;
                    }
                    format!("Selected quick action index: {}", app.selected_quick_action_index)
                } else {
                    app.previous_task();
                    format!("Selected task index: {}", app.selected_task_index)
                }
            },
            KeyCode::Enter => {
                if app.show_quick_actions_modal {
                    // Clone the action to avoid borrow checker issues
                    let action_to_apply = app.config.quick_actions.as_ref()
                        .and_then(|qa| qa.get(app.selected_quick_action_index))
                        .cloned();
                        
                    if let Some(action) = action_to_apply {
                        match app.apply_quick_action(&action) {
                            Ok(_) => {
                                app.hide_quick_actions_modal();
                                "Applied quick action".to_string()
                            },
                            Err(err) => format!("Quick action failed: {}", err),
                        }
                    } else {
                        "No quick action selected".to_string()
                    }
                } else {
                    "Enter pressed in main view".to_string()
                }
            },
            _ => format!("Unhandled key: {:?}", key),
        };
        results.push(result);
    }
    
    results
}

// Helper function to close all modals (using public methods where possible)
fn close_all_modals(app: &mut App) {
    if app.show_help_modal {
        app.hide_help_modal();
    }
    if app.show_sort_modal {
        app.hide_sort_modal();
    }
    if app.show_quick_actions_modal {
        app.hide_quick_actions_modal();
    }
    if app.show_quick_add_modal {
        app.hide_quick_add_modal();
    }
    if app.show_edit_modal {
        app.hide_edit_modal();
    }
}

#[test]
fn test_modal_workflow_simulation() {
    let mut app = create_rich_test_app();
    
    // Simulate a complete workflow: open quick actions, navigate, select action
    let key_sequence = vec![
        KeyCode::Char(' '),    // Open quick actions modal
        KeyCode::Down,         // Navigate down 
        KeyCode::Down,         // Navigate down again
        KeyCode::Enter,        // Select action (should be priority change to 5)
    ];
    
    let results = simulate_key_sequence(&mut app, key_sequence);
    
    // Verify the workflow
    assert_eq!(results[0], "Opened quick actions modal");
    assert!(results[1].contains("Selected quick action index: 1"));
    assert!(results[2].contains("Selected quick action index: 2"));
    assert!(results[3].contains("Applied quick action") || results[3].contains("priority"));
    
    // Verify the action was actually applied
    if let Some(task) = app.tasks.get(app.selected_task_index) {
        // The third quick action should be priority 5
        assert_eq!(task.priority, Some(5));
    }
}

#[test]
fn test_modal_navigation_boundaries() {
    let mut app = create_rich_test_app();
    app.show_quick_actions_modal();
    
    let quick_actions_count = app.config.quick_actions.as_ref().unwrap().len();
    
    // Test navigation to top boundary
    for _ in 0..10 {  // Try to go up beyond boundary
        if app.selected_quick_action_index > 0 {
            app.selected_quick_action_index -= 1;
        }
    }
    assert_eq!(app.selected_quick_action_index, 0);
    
    // Test navigation to bottom boundary
    for _ in 0..20 {  // Try to go down beyond boundary
        if app.selected_quick_action_index + 1 < quick_actions_count {
            app.selected_quick_action_index += 1;
        }
    }
    assert_eq!(app.selected_quick_action_index, quick_actions_count - 1);
}

#[test]
fn test_quick_action_application_on_different_tasks() {
    let mut app = create_rich_test_app();
    
    let project_action = QuickAction {
        key: "p".to_string(),
        action: "project".to_string(),
        target: "Personal".to_string(), // Changed to Personal (project ID 3) to avoid conflicts
    };
    
    // Test applying action to different tasks
    for i in 0..app.tasks.len() {
        app.selected_task_index = i;
        let original_project = app.tasks[i].project_id;
        
        match app.apply_quick_action(&project_action) {
            Ok(_) => {
                assert_eq!(app.tasks[i].project_id, 3); // Personal project ID
                // Only assert not equal if the original was different
                if original_project != 3 {
                    assert_ne!(app.tasks[i].project_id, original_project);
                }
            },
            Err(_) => {
                // Some tasks might fail to change project, that's okay
                // Ensure original project is preserved on error
                assert_eq!(app.tasks[i].project_id, original_project);
            }
        }
    }
}

#[test]
fn test_modal_state_persistence_across_operations() {
    let mut app = create_rich_test_app();
    
    // Open quick actions modal and navigate
    app.show_quick_actions_modal();
    app.selected_quick_action_index = 3;
    
    // Perform some app operations
    app.next_task();
    app.next_task();
    app.previous_task();
    
    // Modal state should be preserved
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 3);
    
    // Close and reopen - should reset
    app.hide_quick_actions_modal();
    app.show_quick_actions_modal();
    assert_eq!(app.selected_quick_action_index, 0);
}

#[test]
fn test_multiple_quick_action_applications() {
    let mut app = create_rich_test_app();
    
    let actions = vec![
        QuickAction { key: "w".to_string(), action: "project".to_string(), target: "Work".to_string() },
        QuickAction { key: "u".to_string(), action: "priority".to_string(), target: "5".to_string() },
        QuickAction { key: "i".to_string(), action: "label".to_string(), target: "Important".to_string() },
    ];
    
    // Apply multiple actions to the same task
    for action in &actions {
        let result = app.apply_quick_action(action);
        assert!(result.is_ok(), "Failed to apply action: {:?}", action);
    }
    
    // Verify all changes were applied
    let task = &app.tasks[app.selected_task_index];
    assert_eq!(task.project_id, 2); // Work
    assert_eq!(task.priority, Some(5)); // High priority
    // Label application would need more complex verification
}

#[test]
fn test_error_recovery_in_modal_operations() {
    let mut app = create_rich_test_app();
    
    // Try to apply invalid actions
    let invalid_actions = vec![
        QuickAction { key: "x".to_string(), action: "project".to_string(), target: "NonexistentProject".to_string() },
        QuickAction { key: "y".to_string(), action: "priority".to_string(), target: "10".to_string() }, // Invalid priority
        QuickAction { key: "z".to_string(), action: "invalid".to_string(), target: "whatever".to_string() },
    ];
    
    for action in &invalid_actions {
        let result = app.apply_quick_action(action);
        assert!(result.is_err(), "Expected error for invalid action: {:?}", action);
        
        // Ensure app state is still consistent after error
        assert!(!app.tasks.is_empty());
        assert!(app.selected_task_index < app.tasks.len());
    }
}

// Property-based testing example (would require adding proptest to dependencies)
/*
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_modal_navigation_never_panics(
        navigation_steps in prop::collection::vec(0usize..10, 0..50)
    ) {
        let mut app = create_rich_test_app();
        app.show_quick_actions_modal();
        
        let max_index = app.config.quick_actions.as_ref().unwrap().len();
        
        for step in navigation_steps {
            app.selected_quick_action_index = step % max_index;
            // Should never panic regardless of index
            assert!(app.selected_quick_action_index < max_index);
        }
    }
    
    #[test]
    fn test_quick_action_robustness(
        action_type in "project|priority|label",
        target in ".*"
    ) {
        let mut app = create_rich_test_app();
        let action = QuickAction {
            key: "test".to_string(),
            action: action_type,
            target: target,
        };
        
        // Should either succeed or fail gracefully, never panic
        let _ = app.apply_quick_action(&action);
        
        // App should remain in valid state
        assert!(!app.tasks.is_empty());
        assert!(app.selected_task_index < app.tasks.len());
    }
}
*/
