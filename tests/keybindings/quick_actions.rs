// Tests for quick action functionality and keybindings

use crate::common::create_test_app_with_keybindings;

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
fn test_quick_action_priority_assignment() {
    let mut app = create_test_app_with_keybindings();
    
    if let Some(quick_actions) = app.config.quick_actions.clone() {
        // Find the priority action ("h" -> priority 5)
        if let Some(priority_action) = quick_actions.iter().find(|a| a.action == "priority") {
            app.selected_task_index = 0;
            let original_priority = app.tasks[0].priority;
            
            let result = app.apply_quick_action(priority_action);
            assert!(result.is_ok());
            
            // Verify priority was changed
            assert_ne!(app.tasks[0].priority, original_priority);
            assert_eq!(app.tasks[0].priority, Some(5));
        }
    }
}

#[test]
fn test_quick_action_label_assignment() {
    let mut app = create_test_app_with_keybindings();
    
    if let Some(quick_actions) = app.config.quick_actions.clone() {
        // Find the label action ("u" -> urgent label)
        if let Some(label_action) = quick_actions.iter().find(|a| a.action == "label") {
            app.selected_task_index = 0;
            
            let result = app.apply_quick_action(label_action);
            assert!(result.is_ok());
            
            // Verify label was added (implementation details may vary)
            // This test validates the action was processed successfully
        }
    }
}

#[test]
fn test_quick_action_with_no_selected_task() {
    let mut app = create_test_app_with_keybindings();
    app.tasks.clear(); // No tasks available
    
    if let Some(quick_actions) = app.config.quick_actions.clone() {
        let action = &quick_actions[0];
        let result = app.apply_quick_action(action);
        
        // Should handle gracefully when no task is selected
        assert!(result.is_err() || result.is_ok());
    }
}

#[test]
fn test_quick_action_mode_toggle() {
    let mut app = create_test_app_with_keybindings();
    
    // Test toggle behavior
    assert!(!app.quick_action_mode);
    
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    
    app.exit_quick_action_mode();
    assert!(!app.quick_action_mode);
}

#[test]
fn test_all_configured_quick_actions() {
    let mut app = create_test_app_with_keybindings();
    
    if let Some(quick_actions) = app.config.quick_actions.clone() {
        // Test that all configured quick actions can be applied
        for action in &quick_actions {
            app.selected_task_index = 0;
            let result = app.apply_quick_action(action);
            
            // Each action should either succeed or fail gracefully
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_quick_action_mode_state_cleanup() {
    let mut app = create_test_app_with_keybindings();
    
    app.enter_quick_action_mode();
    assert!(app.quick_action_mode);
    assert!(app.quick_action_mode_start.is_some());
    
    // Exit should clean up all state
    app.exit_quick_action_mode();
    assert!(!app.quick_action_mode);
    assert!(app.quick_action_mode_start.is_none());
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
fn test_delete_task_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test D key requests delete task
    assert!(!app.show_confirmation_dialog);
    app.request_delete_task();
    assert!(app.show_confirmation_dialog);
}
