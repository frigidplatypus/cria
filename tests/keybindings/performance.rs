// Tests for performance characteristics and memory safety

use crate::common::{create_test_app_with_keybindings, create_test_app_with_tasks};
use std::time::Instant;

#[test]
fn test_keybinding_performance() {
    let mut app = create_test_app_with_tasks(1000);
    
    // Test navigation performance
    let start = Instant::now();
    for _ in 0..100 {
        app.next_task();
    }
    let navigation_time = start.elapsed();
    
    // Should complete quickly even with many tasks
    assert!(navigation_time.as_millis() < 100, 
        "Navigation took too long: {:?}", navigation_time);
    
    // Test filtering performance
    let start = Instant::now();
    for _ in 0..10 {
        app.cycle_task_filter();
    }
    let filter_time = start.elapsed();
    
    assert!(filter_time.as_millis() < 500,
        "Filtering took too long: {:?}", filter_time);
    
    // Test modal operations don't scale with task count
    let start = Instant::now();
    for _ in 0..50 {
        app.show_help_modal();
        app.hide_help_modal();
        app.show_project_picker();
        app.hide_project_picker();
    }
    let modal_time = start.elapsed();
    
    assert!(modal_time.as_millis() < 100,
        "Modal operations took too long: {:?}", modal_time);
}

#[test]
fn test_memory_safety_and_cleanup() {
    let mut app = create_test_app_with_keybindings();
    
    // Test that repeated operations don't leak memory
    // (In a real scenario, you'd use tools like valgrind or AddressSanitizer)
    
    // Repeatedly open/close modals
    for _ in 0..100 {
        app.show_quick_add_modal();
        app.hide_quick_add_modal();
        app.show_edit_modal();
        app.hide_edit_modal();
        app.show_help_modal();
        app.hide_help_modal();
    }
    
    // Verify all modals are properly closed
    assert!(!app.show_quick_add_modal);
    assert!(!app.show_edit_modal);
    assert!(!app.show_help_modal);
    assert!(!app.show_project_picker);
    assert!(!app.show_filter_picker);
    
    // Test that input buffers are properly cleared
    app.show_quick_add_modal();
    for _ in 0..100 {
        app.add_char_to_quick_add('x');
    }
    app.hide_quick_add_modal();
    assert_eq!(app.quick_add_input, ""); // Should be cleared
    
    // Test navigation doesn't accumulate state
    let initial_index = app.selected_task_index;
    for _ in 0..1000 {
        app.next_task();
        app.previous_task();
    }
    // Should be back to start (modulo task count)
    assert_eq!(app.selected_task_index, initial_index);
}

#[test]
fn test_large_dataset_performance() {
    let mut app = create_test_app_with_tasks(10000);
    
    // Test operations scale well with large datasets
    let start = Instant::now();
    
    // Navigate through many tasks
    for _ in 0..1000 {
        app.next_task();
    }
    
    // Jump operations
    app.jump_to_top();
    app.jump_to_bottom();
    
    // Task operations
    app.toggle_task_completion();
    app.toggle_star_selected_task();
    
    let total_time = start.elapsed();
    
    // Operations should complete in reasonable time even with 10k tasks
    assert!(total_time.as_millis() < 1000, 
        "Large dataset operations took too long: {:?}", total_time);
}

#[test]
fn test_rapid_operations_performance() {
    let mut app = create_test_app_with_keybindings();
    
    let start = Instant::now();
    
    // Simulate rapid user input
    for _ in 0..1000 {
        app.next_task();
        app.previous_task();
        app.toggle_info_pane();
        app.toggle_info_pane(); // Toggle back
    }
    
    let rapid_ops_time = start.elapsed();
    
    // Rapid operations should be handled efficiently
    assert!(rapid_ops_time.as_millis() < 500,
        "Rapid operations took too long: {:?}", rapid_ops_time);
}

#[test]
fn test_modal_switching_performance() {
    let mut app = create_test_app_with_keybindings();
    
    let start = Instant::now();
    
    // Test modal switching performance
    for _ in 0..100 {
        app.show_help_modal();
        app.show_quick_add_modal();
        app.show_edit_modal();
        app.show_project_picker();
        app.show_filter_picker();
        app.close_all_modals();
    }
    
    let modal_time = start.elapsed();
    
    // Modal operations should be fast
    assert!(modal_time.as_millis() < 200,
        "Modal switching took too long: {:?}", modal_time);
}

#[test]
fn test_state_consistency_under_load() {
    let mut app = create_test_app_with_tasks(100);
    
    // Perform many operations and ensure state remains consistent
    for i in 0..1000 {
        app.next_task();
        if i % 10 == 0 {
            app.toggle_task_completion();
        }
        if i % 15 == 0 {
            app.show_help_modal();
            app.hide_help_modal();
        }
        if i % 20 == 0 {
            app.cycle_task_filter();
        }
    }
    
    // State should be consistent
    assert!(app.selected_task_index < app.tasks.len());
    assert!(!app.show_help_modal);
    assert!(!app.show_quick_add_modal);
    assert!(!app.show_edit_modal);
}

#[test]
fn test_memory_efficiency() {
    // Test that the app doesn't accumulate unnecessary state
    let mut app = create_test_app_with_keybindings();
    
    let initial_task_count = app.tasks.len();
    
    // Perform many modal operations
    for _ in 0..50 {
        app.show_quick_add_modal();
        app.add_char_to_quick_add('t');
        app.add_char_to_quick_add('e');
        app.add_char_to_quick_add('s');
        app.add_char_to_quick_add('t');
        app.hide_quick_add_modal();
    }
    
    // Task count should remain the same
    assert_eq!(app.tasks.len(), initial_task_count);
    
    // Input should be cleared
    assert_eq!(app.quick_add_input, "");
}
