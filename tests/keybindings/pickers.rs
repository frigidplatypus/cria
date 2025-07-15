// Tests for picker-related keybindings (project picker, filter picker, etc.)

use crate::common::create_test_app_with_keybindings;

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
fn test_filter_picker_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    // Test f key shows filter picker
    app.show_filter_picker();
    assert!(app.show_filter_picker);
    
    app.hide_filter_picker();
    assert!(!app.show_filter_picker);
}

#[test]
fn test_filter_cycling_keybinding() {
    let mut app = create_test_app_with_keybindings();
    
    let initial_filter = app.task_filter.clone();
    app.cycle_task_filter();
    
    // Should have changed to a different filter
    assert_ne!(app.task_filter, initial_filter);
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
fn test_picker_exclusivity() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that showing one picker closes others
    app.show_project_picker();
    assert!(app.show_project_picker);
    
    app.show_filter_picker();
    assert!(app.show_filter_picker);
    assert!(!app.show_project_picker); // Should be closed
}

#[test]
fn test_picker_modal_interaction() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that pickers close when modals are shown
    app.show_project_picker();
    assert!(app.show_project_picker);
    
    app.show_help_modal();
    assert!(app.show_help_modal);
    assert!(!app.show_project_picker); // Should be closed by modal
}

#[test]
fn test_filter_cycling_through_all_states() {
    let mut app = create_test_app_with_keybindings();
    
    let initial_filter = app.task_filter.clone();
    let mut seen_filters = vec![initial_filter.clone()];
    
    // Cycle through filters to test all states
    for _ in 0..10 { // Arbitrary limit to prevent infinite loop
        app.cycle_task_filter();
        let current_filter = app.task_filter.clone();
        
        if seen_filters.contains(&current_filter) {
            break; // We've cycled back to a seen state
        }
        seen_filters.push(current_filter);
    }
    
    // Should have at least 2 different filter states
    assert!(seen_filters.len() >= 2, "Filter cycling should have multiple states");
}

#[test]
fn test_picker_state_initialization() {
    let app = create_test_app_with_keybindings();
    
    // All pickers should start hidden
    assert!(!app.show_project_picker);
    assert!(!app.show_filter_picker);
}

#[test]
fn test_toggle_operations() {
    let mut app = create_test_app_with_keybindings();
    
    // Test info pane toggle consistency
    let initial_info_state = app.show_info_pane;
    
    // Multiple toggles should return to original state
    app.toggle_info_pane();
    app.toggle_info_pane();
    assert_eq!(app.show_info_pane, initial_info_state);
    
    // Test with different starting states
    app.show_info_pane = true;
    app.toggle_info_pane();
    assert!(!app.show_info_pane);
    
    app.toggle_info_pane();
    assert!(app.show_info_pane);
}
