// Tests for edge cases, error handling, and robustness

use crate::common::{create_test_app_with_keybindings, sample_task, KeyEventSimulator};
use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::state::App;

#[test]
fn test_keybinding_edge_cases() {
    let mut app = create_test_app_with_keybindings();
    
    // Test navigation with empty task list
    app.tasks.clear();
    app.selected_task_index = 0;
    app.next_task();
    assert_eq!(app.selected_task_index, 0); // Should stay at 0
    
    app.previous_task();
    assert_eq!(app.selected_task_index, 0); // Should stay at 0
    
    // Test with out-of-bounds task index
    app.tasks = vec![sample_task(1, false)];
    app.selected_task_index = 10; // Out of bounds
    let task_id = app.toggle_task_completion();
    assert!(task_id.is_none()); // Should return None for invalid index
    
    // Test rapid modal switching
    app.show_help_modal();
    app.show_project_picker(); // Should close help modal
    assert!(!app.show_help_modal);
    assert!(app.show_project_picker);
    
    app.show_quick_add_modal(); // Should close project picker
    assert!(!app.show_project_picker);
    assert!(app.show_quick_add_modal);
}

#[test]
fn test_error_handling_robustness() {
    let mut app = create_test_app_with_keybindings();
    
    // Test operations on empty task list
    app.tasks.clear();
    assert!(app.toggle_task_completion().is_none());
    assert!(app.toggle_star_selected_task().is_none());
    
    // Test with invalid selected index
    app.tasks = vec![sample_task(1, false)];
    app.selected_task_index = usize::MAX;
    assert!(app.toggle_task_completion().is_none());
    
    // Test quick action with missing config
    app.config.quick_actions = None;
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode); // Should still work
    app.exit_quick_action_mode();
    
    // Test operations that should never panic
    app.cycle_task_filter();
    app.toggle_info_pane();
    app.show_help_modal();
    app.hide_help_modal();
    
    // All operations should complete without panicking
    assert!(true);
}

#[test]
fn test_modifier_key_combinations() {
    let mut app = create_test_app_with_keybindings();
    
    let _simulator = KeyEventSimulator::new();
    
    // Test Ctrl+Z (undo)
    if let Some(_task_id) = app.undo_last_action() {
        // Undo was successful
    } else {
        // Nothing to undo (expected for fresh app)
    }
    
    // Test Ctrl+Y (redo)
    if let Some(_task_id) = app.redo_last_action() {
        // Redo was successful
    } else {
        // Nothing to redo (expected)
    }
    
    // Test that modifiers are properly differentiated
    // (In real implementation, this would be handled by the event loop)
}

#[test]
fn test_boundary_conditions() {
    let mut app = create_test_app_with_keybindings();
    
    // Test with out-of-bounds task index (but not extreme values that cause overflow)
    app.selected_task_index = app.tasks.len(); // One past the end
    app.next_task();
    assert!(app.selected_task_index < app.tasks.len() || app.tasks.is_empty());
    
    // Test navigation at the start
    app.selected_task_index = 0;
    app.previous_task();
    assert!(app.selected_task_index < app.tasks.len() || app.tasks.is_empty());
    
    // Test operations on single task
    app.tasks = vec![sample_task(1, false)];
    app.selected_task_index = 0;
    
    app.next_task();
    assert_eq!(app.selected_task_index, 0); // Should wrap around to 0 for single task
    
    app.previous_task();
    assert_eq!(app.selected_task_index, 0); // Should stay at 0 for single task
}

#[test]
fn test_invalid_task_operations() {
    let mut app = create_test_app_with_keybindings();
    
    // Test operations with no tasks
    app.tasks.clear();
    assert!(app.toggle_task_completion().is_none());
    assert!(app.toggle_star_selected_task().is_none());
    
    // Test operations with invalid index
    app.tasks = vec![sample_task(1, false), sample_task(2, false)];
    app.selected_task_index = 100;
    assert!(app.toggle_task_completion().is_none());
    assert!(app.toggle_star_selected_task().is_none());
}

#[test]
fn test_modal_state_recovery() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that invalid states can be recovered
    app.show_help_modal = true;
    app.show_quick_add_modal = true; // Invalid state - both modals open
    
    // Opening any modal should fix the state
    app.show_project_picker();
    assert!(app.show_project_picker);
    assert!(!app.show_help_modal);
    assert!(!app.show_quick_add_modal);
}

#[test]
fn test_configuration_edge_cases() {
    let mut app = create_test_app_with_keybindings();
    
    // Test with empty quick actions
    app.config.quick_actions = Some(vec![]);
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    app.exit_quick_action_mode();
    
    // Test with None quick actions
    app.config.quick_actions = None;
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    app.exit_quick_action_mode();
}

#[test]
fn test_state_consistency_after_errors() {
    let mut app = create_test_app_with_keybindings();
    
    let initial_state = (
        app.selected_task_index,
        app.show_help_modal,
        app.show_quick_add_modal,
        app.quick_action_mode,
    );
    
    // Perform operations that might cause errors
    app.selected_task_index = usize::MAX;
    app.toggle_task_completion(); // Should fail gracefully
    
    app.tasks.clear();
    app.toggle_star_selected_task(); // Should fail gracefully
    
    // Modal states should be preserved even after task operation failures
    assert_eq!(app.show_help_modal, initial_state.1);
    assert_eq!(app.show_quick_add_modal, initial_state.2);
    assert_eq!(app.quick_action_mode, initial_state.3);
}

#[test]
fn test_rapid_state_changes() {
    let mut app = create_test_app_with_keybindings();
    
    // Test rapid modal switching doesn't break state
    for _ in 0..100 {
        app.show_help_modal();
        app.show_quick_add_modal();
        app.show_project_picker();
        app.close_all_modals();
    }
    
    // Should end in clean state
    assert!(!app.show_help_modal);
    assert!(!app.show_quick_add_modal);
    assert!(!app.show_project_picker);
}

/// Test configuration validation and robustness
#[test]
fn test_config_dependent_keybindings() {
    // Test with minimal config
    let minimal_config = CriaConfig::default();
    let mut app = App::new_with_config(minimal_config, "Inbox".to_string());
    app.tasks = vec![sample_task(1, false)];
    
    // Quick actions should handle missing config gracefully
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    app.exit_quick_action_mode();
    
    // Test with rich config
    let mut rich_config = CriaConfig::default();
    rich_config.quick_actions = Some(vec![
        QuickAction {
            key: "1".to_string(),
            action: "priority".to_string(),
            target: "1".to_string(),
        },
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
    ]);
    
    let mut app_rich = App::new_with_config(rich_config, "Inbox".to_string());
    app_rich.tasks = vec![sample_task(1, false)];
    app_rich.project_map.insert(2, "Work".to_string());
    
    // Quick actions should work with config
    app_rich.enter_quick_action_mode();
    assert!(app_rich.quick_action_mode);
    
    // Apply a quick action
    let action = QuickAction {
        key: "w".to_string(),
        action: "project".to_string(),
        target: "Work".to_string(),
    };
    let result = app_rich.apply_quick_action(&action);
    assert!(result.is_ok());
}
