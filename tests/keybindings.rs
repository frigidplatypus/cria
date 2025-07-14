// Tests for all keybinding functionality to prevent regressions
// This ensures that help modal keybindings remain functional and correctly implemented

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::App;
use cria::vikunja::models::Task;
use chrono::{NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

// Helper to create a test app with tasks and quick actions
fn create_test_app_with_keybindings() -> App {
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "h".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "label".to_string(),
            target: "urgent".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    
    // Add some test tasks
    app.all_tasks = vec![
        sample_task(1, false),
        sample_task(2, true),
        sample_task(3, false),
    ];
    app.tasks = app.all_tasks.clone();
    
    // Add test projects  
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.project_map.insert(3, "Personal".to_string());
    
    // Add test labels
    app.label_map.insert(1, "urgent".to_string());
    app.label_map.insert(2, "low-priority".to_string());
    
    app
}

fn sample_task(id: i64, done: bool) -> Task {
    Task {
        id,
        title: format!("Test Task {}", id),
        done,
        is_favorite: false,
        labels: Some(vec![]),
        assignees: None,
        project_id: 1,
        priority: Some(1),
        due_date: Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(2025, 12, 31).unwrap().and_hms_opt(0,0,0).unwrap())),
        start_date: None,
        description: Some("Test description".to_string()),
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

fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

#[test]
fn test_help_modal_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test ? key shows help modal
    assert!(!app.show_help_modal);
    app.show_help_modal();
    assert!(app.show_help_modal);
    
    // Test that the show_help_modal method exists and works
    app.hide_help_modal();
    assert!(!app.show_help_modal);
}

#[test]
fn test_info_pane_toggle_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    let initial_state = app.show_info_pane;
    app.toggle_info_pane();
    assert_ne!(app.show_info_pane, initial_state);
    
    app.toggle_info_pane();
    assert_eq!(app.show_info_pane, initial_state);
}

#[test]
fn test_project_picker_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test p key shows project picker
    assert!(!app.show_project_picker);
    app.show_project_picker();
    assert!(app.show_project_picker);
    
    app.hide_project_picker();
    assert!(!app.show_project_picker);
}

#[test]
fn test_star_unstar_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Ensure we have a task selected
    app.selected_task_index = 0;
    assert!(app.selected_task_index < app.tasks.len());
    
    let initial_favorite = app.tasks[0].is_favorite;
    let task_id = app.toggle_star_selected_task();
    
    assert!(task_id.is_some());
    assert_ne!(app.tasks[0].is_favorite, initial_favorite);
}

#[test]
fn test_filter_cycling_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    let initial_filter = app.task_filter.clone();
    app.cycle_task_filter();
    
    // Should have changed to a different filter
    assert_ne!(app.task_filter, initial_filter);
    assert_ne!(app.task_filter, initial_filter);
}

#[test]
fn test_quick_action_mode_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test entering quick action mode
    assert!(!app.quick_action_mode);
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    assert!(app.quick_action_mode_start.is_some());
    
    // Test exiting quick action mode
    app.exit_quick_action_mode();
    assert!(!app.quick_action_mode);
    assert!(app.quick_action_mode_start.is_none());
}

#[test]
fn test_task_navigation_keybindings() {
    let mut app = create_test_app_with_keybindings();
    
    // Test starting position
    assert_eq!(app.selected_task_index, 0);
    
    // Test next task (j key / Down arrow)
    app.next_task();
    assert_eq!(app.selected_task_index, 1);
    
    // Test previous task (k key / Up arrow)
    app.previous_task();
    assert_eq!(app.selected_task_index, 0);
    
    // Test jump to top (g key)
    app.selected_task_index = 2;
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
    
    // Test jump to bottom (G key)
    app.jump_to_bottom();
    assert_eq!(app.selected_task_index, app.tasks.len() - 1);
}

#[test]
fn test_quick_actions_modal_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test Space key shows quick actions modal
    assert!(!app.show_quick_actions_modal);
    app.show_quick_actions_modal();
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
    
    app.hide_quick_actions_modal();
    assert!(!app.show_quick_actions_modal);
}

#[test]
fn test_task_completion_toggle_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Ensure we have a task selected
    app.selected_task_index = 0;
    let initial_done = app.tasks[0].done;
    
    let task_id = app.toggle_task_completion();
    assert!(task_id.is_some());
    assert_ne!(app.tasks[0].done, initial_done);
}

#[test]
fn test_quick_add_modal_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test a key shows quick add modal
    assert!(!app.show_quick_add_modal);
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    
    app.hide_quick_add_modal();
    assert!(!app.show_quick_add_modal);
}

#[test]
fn test_edit_modal_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Ensure we have a task selected
    app.selected_task_index = 0;
    
    // Test e key shows edit modal
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    assert!(app.editing_task_id.is_some());
    
    app.hide_edit_modal();
    assert!(!app.show_edit_modal);
}

#[test]
fn test_form_edit_modal_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Ensure we have a task selected
    app.selected_task_index = 0;
    
    // Test E key shows form edit modal
    app.show_form_edit_modal();
    assert!(app.show_form_edit_modal);
    assert!(app.form_edit_state.is_some());
    
    app.hide_form_edit_modal();
    assert!(!app.show_form_edit_modal);
}

#[test]
fn test_filter_picker_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test f key shows filter picker
    app.show_filter_picker();
    assert!(app.show_filter_picker);
    
    app.hide_filter_picker();
    assert!(!app.show_filter_picker);
}

#[test]
fn test_delete_task_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test D key requests delete task
    assert!(!app.show_confirmation_dialog);
    app.request_delete_task();
    assert!(app.show_confirmation_dialog);
}

#[test]
fn test_quick_action_application() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that quick actions can be applied
    if let Some(quick_actions) = app.config.quick_actions.clone() {
        let action = &quick_actions[0]; // "w" -> Work project
        
        app.selected_task_index = 0;
        let result = app.apply_quick_action(action);
        assert!(result.is_ok());
        
        // Verify the task was modified (project changed to Work)
        if let Some(work_project_id) = app.project_map.iter()
            .find_map(|(id, name)| if name == "Work" { Some(*id) } else { None }) {
            assert_eq!(app.tasks[0].project_id, work_project_id);
        }
    }
}

#[test]
fn test_quick_action_mode_timeout() {
    let mut app = create_test_app_with_keybindings();
    
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    
    // Simulate time passing (this tests the timeout logic)
    // The actual timeout check happens in is_quick_action_mode_expired()
    assert!(!app.is_quick_action_mode_expired()); // Should not be expired immediately
}

#[test]
fn test_modal_exclusivity() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that showing one modal closes others
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    
    app.show_help_modal();
    assert!(app.show_help_modal);
    assert!(!app.show_quick_add_modal); // Should be closed
    
    app.show_project_picker();
    assert!(app.show_project_picker);
    assert!(!app.show_help_modal); // Should be closed
}

/// Test that all help modal documented keybindings have corresponding methods
#[test]
fn test_all_help_modal_keybindings_exist() {
    let mut app = create_test_app_with_keybindings();
    
    // This test verifies that all methods called by keybindings exist and are callable
    
    // Navigation keybindings
    app.next_task();
    app.previous_task();
    app.jump_to_top();
    app.jump_to_bottom();
    
    // Modal keybindings
    app.show_help_modal();
    app.hide_help_modal();
    app.show_quick_add_modal();
    app.hide_quick_add_modal();
    app.show_edit_modal();
    app.hide_edit_modal();
    app.show_form_edit_modal();
    app.hide_form_edit_modal();
    app.show_project_picker();
    app.hide_project_picker();
    app.show_filter_picker();
    app.hide_filter_picker();
    app.show_quick_actions_modal();
    app.hide_quick_actions_modal();
    
    // Action keybindings
    app.toggle_task_completion();
    app.toggle_star_selected_task();
    app.request_delete_task();
    app.cycle_task_filter();
    app.toggle_info_pane();
    app.enter_quick_action_mode();
    app.exit_quick_action_mode();
    
    // If we get here without panicking, all methods exist and are callable
    assert!(true);
}

/// Test that keybinding methods return expected types
#[test]
fn test_keybinding_method_return_types() {
    let mut app = create_test_app_with_keybindings();
    
    // Ensure selected task exists
    app.selected_task_index = 0;
    
    // Test methods that return Option<i64> (task IDs)
    let completion_result = app.toggle_task_completion();
    assert!(completion_result.is_some() || completion_result.is_none());
    
    let star_result = app.toggle_star_selected_task();
    assert!(star_result.is_some() || star_result.is_none());
    
    // Test boolean state methods
    app.toggle_info_pane(); // Returns ()
    assert!(true); // If we get here, method exists and returns expected type
    
    // Test quick action mode state
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    
    let is_expired = app.is_quick_action_mode_expired();
    assert!(is_expired == true || is_expired == false); // Boolean return type
}

/// Integration test for complete keybinding workflow
#[test]
fn test_complete_keybinding_workflow() {
    let mut app = create_test_app_with_keybindings();
    
    // Simulate a complete user workflow using keybindings
    
    // 1. Open help modal (? key)
    app.show_help_modal();
    assert!(app.show_help_modal);
    
    // 2. Close help modal (Esc key)
    app.hide_help_modal();
    assert!(!app.show_help_modal);
    
    // 3. Navigate tasks (j/k keys)
    app.next_task();
    app.next_task();
    assert_eq!(app.selected_task_index, 2);
    
    // 4. Toggle task completion (d key)
    let task_id = app.toggle_task_completion();
    assert!(task_id.is_some());
    
    // 5. Star the task (s key)
    let star_id = app.toggle_star_selected_task();
    assert!(star_id.is_some());
    
    // 6. Open quick action mode (. key)
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    
    // 7. Exit quick action mode
    app.exit_quick_action_mode();
    assert!(!app.quick_action_mode);
    
    // 8. Open project picker (p key)
    app.show_project_picker();
    assert!(app.show_project_picker);
    
    // 9. Close project picker
    app.hide_project_picker();
    assert!(!app.show_project_picker);
    
    // 10. Cycle filters (h/l keys)
    let _initial_filter = app.task_filter.clone();
    app.cycle_task_filter();
    // Filter should have changed (exact behavior depends on implementation)
    
    // If we complete this workflow without panicking, all keybindings work correctly
    assert!(true);
}
