// Comprehensive tests for the Undo/Redo system
// Tests for UndoableAction, undo_stack, redo_stack, and all undo/redo operations

use cria::config::CriaConfig;
use cria::tui::app::state::App;
use cria::tui::app::undoable_action::UndoableAction;
use cria::vikunja::models::Task;
use chrono::{NaiveDate, TimeZone, Utc};

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
fn test_undo_stack_initialization() {
    let app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    assert!(app.undo_stack.is_empty());
    assert!(app.redo_stack.is_empty());
}

#[test]
fn test_task_completion_undo_redo() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Initial state: task is not done
    assert!(!app.tasks[0].done);
    assert!(app.undo_stack.is_empty());
    assert!(app.redo_stack.is_empty());
    
    // Toggle completion (should add to undo stack)
    let task_id = app.toggle_task_completion().unwrap();
    assert_eq!(task_id, 1);
    assert!(app.tasks[0].done);
    assert_eq!(app.undo_stack.len(), 1);
    assert!(app.redo_stack.is_empty());
    
    // Verify undo stack contains correct action
    if let UndoableAction::TaskCompletion { task_id: undo_task_id, previous_state } = &app.undo_stack[0] {
        assert_eq!(*undo_task_id, 1);
        assert!(!previous_state); // Was not done before toggle
    } else {
        panic!("Expected TaskCompletion action in undo stack");
    }
    
    // Undo the completion
    let undone_task_id = app.undo_last_action().unwrap();
    assert_eq!(undone_task_id, 1);
    assert!(!app.tasks[0].done); // Back to original state
    assert!(app.undo_stack.is_empty());
    assert_eq!(app.redo_stack.len(), 1);
    
    // Verify redo stack contains correct action
    if let UndoableAction::TaskCompletion { task_id: redo_task_id, previous_state } = &app.redo_stack[0] {
        assert_eq!(*redo_task_id, 1);
        assert!(*previous_state); // Was done after toggle
    } else {
        panic!("Expected TaskCompletion action in redo stack");
    }
    
    // Redo the completion
    let redone_task_id = app.redo_last_action().unwrap();
    assert_eq!(redone_task_id, 1);
    assert!(app.tasks[0].done); // Back to completed state
    assert_eq!(app.undo_stack.len(), 1);
    assert!(app.redo_stack.is_empty());
}

#[test]
fn test_task_deletion_undo_redo() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    let task = sample_task(1, false);
    app.tasks.push(task.clone());
    
    // Initial state
    assert_eq!(app.tasks.len(), 1);
    assert!(app.undo_stack.is_empty());
    assert!(app.redo_stack.is_empty());
    
    // Delete task
    app.request_delete_task();
    app.confirm_action();
    assert!(app.tasks.is_empty());
    assert_eq!(app.undo_stack.len(), 1);
    
    // Verify undo stack contains correct action
    if let UndoableAction::TaskDeletion { task, position } = &app.undo_stack[0] {
        assert_eq!(task.id, 1);
        assert_eq!(*position, 0);
    } else {
        panic!("Expected TaskDeletion action in undo stack");
    }
    
    // Undo deletion
    let restored_task_id = app.undo_last_action().unwrap();
    assert_eq!(restored_task_id, 1);
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].id, 1);
    assert!(app.undo_stack.is_empty());
    assert_eq!(app.redo_stack.len(), 1);
    
    // Redo deletion
    let deleted_task_id = app.redo_last_action().unwrap();
    assert_eq!(deleted_task_id, 1);
    assert!(app.tasks.is_empty());
    assert_eq!(app.undo_stack.len(), 1);
    assert!(app.redo_stack.is_empty());
}

#[test]
fn test_task_edit_undo_redo() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    let mut task = sample_task(1, false);
    task.title = "Original Title".to_string();
    app.tasks.push(task.clone());
    
    // Edit task title
    if let Some(selected_task) = app.tasks.get_mut(0) {
        let old_task = selected_task.clone();
        selected_task.title = "New Title".to_string();
        app.add_to_undo_stack(UndoableAction::TaskEdit { 
            task_id: 1, 
            previous_task: old_task 
        });
    }
    
    assert_eq!(app.tasks[0].title, "New Title");
    assert_eq!(app.undo_stack.len(), 1);
    
    // Undo edit
    let undone_task_id = app.undo_last_action().unwrap();
    assert_eq!(undone_task_id, 1);
    assert_eq!(app.tasks[0].title, "Original Title");
    assert_eq!(app.redo_stack.len(), 1);
    
    // Redo edit
    let redone_task_id = app.redo_last_action().unwrap();
    assert_eq!(redone_task_id, 1);
    assert_eq!(app.tasks[0].title, "New Title");
}

#[test]
fn test_undo_redo_empty_stacks() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    
    // Try to undo with empty stack
    assert!(app.undo_last_action().is_none());
    
    // Try to redo with empty stack
    assert!(app.redo_last_action().is_none());
}

#[test]
fn test_undo_stack_size_limit() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Perform more than the undo stack limit (50) actions
    for _ in 0..60 {
        app.toggle_task_completion();
    }
    
    // Undo stack should be limited to 50 items
    assert_eq!(app.undo_stack.len(), 50);
}

#[test]
fn test_redo_stack_size_limit() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Perform 60 actions and then undo them all
    for _ in 0..60 {
        app.toggle_task_completion();
    }
    
    // Now undo 60 times (this should fill the redo stack beyond limit)
    for _ in 0..60 {
        if app.undo_last_action().is_none() {
            break;
        }
    }
    
    // Redo stack should be limited to 50 items
    assert_eq!(app.redo_stack.len(), 50);
}

#[test]
fn test_new_action_clears_redo_stack() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.tasks.push(sample_task(2, false));
    
    // Perform action -> undo -> perform new action
    app.toggle_task_completion(); // First action
    assert_eq!(app.undo_stack.len(), 1);
    assert_eq!(app.redo_stack.len(), 0);
    
    app.undo_last_action(); // Undo first action
    assert_eq!(app.undo_stack.len(), 0);
    assert_eq!(app.redo_stack.len(), 1);
    
    // Select different task and perform new action
    app.selected_task_index = 1;
    app.toggle_task_completion(); // New action should clear redo stack
    assert_eq!(app.undo_stack.len(), 1);
    assert_eq!(app.redo_stack.len(), 0); // Redo stack should be cleared
}

#[test]
fn test_multiple_undo_redo_sequence() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.tasks.push(sample_task(2, false));
    
    // Perform multiple actions
    app.selected_task_index = 0;
    app.toggle_task_completion(); // Action 1
    
    // Manually add a task edit action for testing
    if let Some(task) = app.tasks.get_mut(0) {
        let old_task = task.clone();
        task.title = "Modified Task".to_string();
        app.add_to_undo_stack(UndoableAction::TaskEdit { 
            task_id: 1, 
            previous_task: old_task 
        });
    }
    
    app.selected_task_index = 1;
    app.toggle_task_completion(); // Action 3
    
    assert_eq!(app.undo_stack.len(), 3);
    assert_eq!(app.redo_stack.len(), 0);
    
    // Undo all actions
    app.undo_last_action(); // Undo action 3
    app.undo_last_action(); // Undo action 2
    app.undo_last_action(); // Undo action 1
    
    assert_eq!(app.undo_stack.len(), 0);
    assert_eq!(app.redo_stack.len(), 3);
    
    // Verify final state
    assert!(!app.tasks[0].done);
    assert_eq!(app.tasks[0].title, "Task 1"); // Back to original title
    assert!(!app.tasks[1].done);
    
    // Redo all actions
    app.redo_last_action(); // Redo action 1
    app.redo_last_action(); // Redo action 2
    app.redo_last_action(); // Redo action 3
    
    assert_eq!(app.undo_stack.len(), 3);
    assert_eq!(app.redo_stack.len(), 0);
    
    // Verify final state
    assert!(app.tasks[0].done);
    assert_eq!(app.tasks[0].title, "Modified Task");
    assert!(app.tasks[1].done);
}

#[test]
fn test_undo_redo_with_task_deletion_and_restoration() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.tasks.push(sample_task(2, false));
    app.tasks.push(sample_task(3, false));
    
    // Delete middle task
    app.selected_task_index = 1;
    app.request_delete_task();
    app.confirm_action();
    
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[0].id, 1);
    assert_eq!(app.tasks[1].id, 3); // Task 2 was deleted
    
    // Undo deletion
    app.undo_last_action();
    
    assert_eq!(app.tasks.len(), 3);
    assert_eq!(app.tasks[0].id, 1);
    assert_eq!(app.tasks[1].id, 2); // Task 2 is restored at original position
    assert_eq!(app.tasks[2].id, 3);
    
    // Redo deletion
    app.redo_last_action();
    
    assert_eq!(app.tasks.len(), 2);
    assert_eq!(app.tasks[0].id, 1);
    assert_eq!(app.tasks[1].id, 3); // Task 2 is deleted again
}

#[test]
fn test_undoable_action_debug_display() {
    let task = sample_task(1, false);
    
    let completion_action = UndoableAction::TaskCompletion { task_id: 1, previous_state: false };
    let deletion_action = UndoableAction::TaskDeletion { task: task.clone(), position: 0 };
    let creation_action = UndoableAction::TaskCreation { task_id: 1 };
    let edit_action = UndoableAction::TaskEdit { task_id: 1, previous_task: task };
    
    // These should not panic and should produce reasonable debug output
    let _ = format!("{:?}", completion_action);
    let _ = format!("{:?}", deletion_action);
    let _ = format!("{:?}", creation_action);
    let _ = format!("{:?}", edit_action);
}

#[test]
fn test_undo_stack_limit_50() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Add exactly 50 actions
    for _ in 0..50 {
        app.toggle_task_completion();
    }
    
    assert_eq!(app.undo_stack.len(), 50);
    
    // Add one more action - should still be 50 (oldest removed)
    app.toggle_task_completion();
    assert_eq!(app.undo_stack.len(), 50);
}

#[test]
fn test_redo_stack_limit_50() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Add 50 actions
    for _ in 0..50 {
        app.toggle_task_completion();
    }
    
    // Undo all 50 actions
    for _ in 0..50 {
        if app.undo_last_action().is_none() {
            break;
        }
    }
    
    assert_eq!(app.redo_stack.len(), 50);
    
    // Manually add one more to redo stack to test limit
    app.redo_stack.push(UndoableAction::TaskCompletion { task_id: 1, previous_state: true });
    if app.redo_stack.len() > 50 {
        app.redo_stack.remove(0);
    }
    assert_eq!(app.redo_stack.len(), 50);
}

#[test]
fn test_undo_redo_task_creation() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    
    // Simulate task creation by adding task and undo action
    let task = sample_task(1, false);
    app.tasks.push(task);
    app.add_to_undo_stack(UndoableAction::TaskCreation { task_id: 1 });
    
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.undo_stack.len(), 1);
    
    // Undo creation (should remove task)
    let undone_task_id = app.undo_last_action().unwrap();
    assert_eq!(undone_task_id, 1);
    assert_eq!(app.tasks.len(), 0);
    assert_eq!(app.redo_stack.len(), 1);
    
    // Redo creation (should restore task)
    let redone_task_id = app.redo_last_action().unwrap();
    assert_eq!(redone_task_id, 1);
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].id, 1);
}

#[test] 
fn test_undo_redo_clears_opposite_stack() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Test that new actions clear the redo stack
    app.toggle_task_completion(); // Action 1: mark as done
    assert_eq!(app.undo_stack.len(), 1);
    assert_eq!(app.redo_stack.len(), 0);
    
    // Undo the action
    app.undo_last_action(); // Creates redo entry
    assert_eq!(app.undo_stack.len(), 0);
    assert_eq!(app.redo_stack.len(), 1);
    
    // Perform a new action - this should clear the redo stack
    app.toggle_task_completion(); // Action 2: mark as done again
    assert_eq!(app.undo_stack.len(), 1);
    assert_eq!(app.redo_stack.len(), 0); // Should be cleared by add_to_undo_stack
}
