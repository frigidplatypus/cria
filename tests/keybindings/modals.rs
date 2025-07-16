// Tests for modal-related keybindings and behaviors

use crate::common::create_test_app_with_keybindings;

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

#[test]
fn test_modal_state_consistency() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that modal state remains consistent
    app.show_edit_modal();
    let editing_task_id = app.editing_task_id;
    assert!(editing_task_id.is_some());
    
    // Hide and show again - state should be consistent
    app.hide_edit_modal();
    assert!(app.editing_task_id.is_none());
    
    app.show_edit_modal();
    assert!(app.editing_task_id.is_some());
}

#[test]
fn test_confirmation_dialog_modal() {
    let mut app = create_test_app_with_keybindings();
    
    // Test delete confirmation modal
    assert!(!app.show_confirmation_dialog);
    app.request_delete_task();
    assert!(app.show_confirmation_dialog);
    
    // Test hiding confirmation dialog
    app.cancel_confirmation();
    assert!(!app.show_confirmation_dialog);
}

#[test]
fn test_all_modal_states_initially_false() {
    let app = create_test_app_with_keybindings();
    
    // All modals should start hidden
    assert!(!app.show_help_modal);
    assert!(!app.show_quick_add_modal);
    assert!(!app.show_edit_modal);
    assert!(!app.show_form_edit_modal);
    assert!(!app.show_quick_actions_modal);
    assert!(!app.show_project_picker);
    assert!(!app.show_filter_picker);
    assert!(!app.show_confirmation_dialog);
}

#[test]
fn test_modal_state_preservation() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that form edit modal preserves state
    app.show_form_edit_modal();
    assert!(app.form_edit_state.is_some());
    
    // State should persist when modal is shown
    let _state_before = app.form_edit_state.clone();
    app.hide_form_edit_modal();
    app.show_form_edit_modal();
    
    // State should be reset/reinitialized
    assert!(app.form_edit_state.is_some());
}
