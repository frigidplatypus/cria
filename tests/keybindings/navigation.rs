// Tests for task navigation keybindings

use crate::common::{create_test_app_with_keybindings, create_test_app_with_tasks};

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
fn test_navigation_with_empty_task_list() {
    let mut app = create_test_app_with_keybindings();
    app.tasks.clear();
    
    // Test navigation with empty task list
    app.next_task();
    assert_eq!(app.selected_task_index, 0);
    
    app.previous_task();
    assert_eq!(app.selected_task_index, 0);
    
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
    
    app.jump_to_bottom();
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_navigation_performance() {
    let mut app = create_test_app_with_tasks(1000);
    
    // Test navigation performance
    let start = std::time::Instant::now();
    for _ in 0..100 {
        app.next_task();
        app.previous_task();
    }
    let navigation_time = start.elapsed();
    
    // Navigation should be fast
    assert!(navigation_time.as_millis() < 100, 
        "Navigation took too long: {:?}", navigation_time);
}

#[test]
fn test_navigation_wraparound_behavior() {
    let mut app = create_test_app_with_keybindings();
    
    // Test wraparound when going past the end
    app.selected_task_index = app.tasks.len() - 1;
    app.next_task();
    // Should wrap to beginning or stay at end - depends on implementation
    assert!(app.selected_task_index < app.tasks.len());
    
    // Test wraparound when going before the beginning
    app.selected_task_index = 0;
    app.previous_task();
    // Should wrap to end or stay at beginning - depends on implementation
    assert!(app.selected_task_index < app.tasks.len());
}

#[test]
fn test_navigation_boundary_conditions() {
    let mut app = create_test_app_with_keybindings();
    
    // Test navigation at the start - previous_task should wrap to end
    app.selected_task_index = 0;
    app.previous_task();
    // Should wrap around to the last task
    assert_eq!(app.selected_task_index, app.tasks.len() - 1);
    
    // Test navigation at the end - next_task should wrap to start
    app.selected_task_index = app.tasks.len() - 1;
    app.next_task();
    // Should wrap around to the first task
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_jump_operations() {
    let mut app = create_test_app_with_tasks(10);
    
    // Start in the middle
    app.selected_task_index = 5;
    
    // Jump to top
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
    
    // Jump to bottom
    app.jump_to_bottom();
    assert_eq!(app.selected_task_index, app.tasks.len() - 1);
    
    // Jump to top again
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
}
