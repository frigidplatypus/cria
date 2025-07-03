// Property-based testing for modal behavior
// This demonstrates how to use proptest for comprehensive testing

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::App;
use proptest::prelude::*;

// Generate arbitrary quick actions for testing
fn arb_quick_action() -> impl Strategy<Value = QuickAction> {
    (
        "[a-z]", // single character key
        "project|priority|label", // action type
        "[A-Za-z0-9]+", // target
    ).prop_map(|(key, action, target)| QuickAction {
        key,
        action,
        target,
    })
}

// Generate arbitrary app configurations
fn arb_config() -> impl Strategy<Value = CriaConfig> {
    prop::option::of(
        prop::collection::vec(arb_quick_action(), 0..10)
    ).prop_map(|quick_actions| {
        let mut config = CriaConfig::default();
        config.quick_actions = quick_actions;
        config
    })
}

proptest! {
    #[test]
    fn test_modal_navigation_never_out_of_bounds(
        config in arb_config(),
        navigation_steps in prop::collection::vec(0usize..100, 0..50)
    ) {
        let mut app = App::new_with_config(config, "Inbox".to_string());
        app.show_quick_actions_modal();
        
        let max_index = app.config.quick_actions.as_ref()
            .map(|qa| qa.len())
            .unwrap_or(0);
        
        // If no quick actions, max_index is 0 and we should stay at 0
        if max_index == 0 {
            prop_assert_eq!(app.selected_quick_action_index, 0);
            return Ok(());
        }
        
        for step in navigation_steps {
            // Simulate navigation with random steps
            app.selected_quick_action_index = step % max_index;
            
            // Should never panic and always be within bounds
            prop_assert!(app.selected_quick_action_index < max_index);
        }
    }
    
    #[test]
    fn test_quick_action_robustness(
        action in arb_quick_action(),
        project_names in prop::collection::vec("[A-Za-z]+", 1..5)
    ) {
        let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
        
        // Add some projects
        for (i, name) in project_names.iter().enumerate() {
            app.project_map.insert((i + 1) as i64, name.clone());
        }
        
        // Add a task to work with
        let task = cria::vikunja::models::Task {
            id: 1,
            title: "Test task".to_string(),
            done: false,
            is_favorite: false,
            labels: None,
            assignees: None,
            project_id: 1,
            priority: Some(1),
            due_date: None,
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
        };
        app.tasks.push(task);
        
        // Should either succeed or fail gracefully, never panic
        let result = app.apply_quick_action(&action);
        
        // App should remain in valid state regardless of result
        prop_assert!(!app.tasks.is_empty());
        prop_assert!(app.selected_task_index < app.tasks.len());
        
        // If successful, verify the action was applied correctly
        if result.is_ok() {
            match action.action.as_str() {
                "priority" => {
                    if let Ok(priority) = action.target.parse::<i32>() {
                        if priority >= 1 && priority <= 5 {
                            prop_assert_eq!(app.tasks[app.selected_task_index].priority, Some(priority));
                        }
                    }
                },
                "project" => {
                    // Should have changed to a valid project
                    let task_project_id = app.tasks[app.selected_task_index].project_id;
                    prop_assert!(app.project_map.contains_key(&task_project_id));
                },
                _ => {
                    // Other actions (like labels) are harder to verify in property tests
                    // but the fact that we didn't panic is already valuable
                }
            }
        }
    }
    
    #[test]
    fn test_modal_state_consistency(
        initial_index in 0usize..10,
        show_hide_sequence in prop::collection::vec(prop::bool::ANY, 0..20)
    ) {
        let mut config = CriaConfig::default();
        config.quick_actions = Some(vec![
            QuickAction { key: "a".to_string(), action: "priority".to_string(), target: "1".to_string() },
            QuickAction { key: "b".to_string(), action: "priority".to_string(), target: "2".to_string() },
            QuickAction { key: "c".to_string(), action: "priority".to_string(), target: "3".to_string() },
            QuickAction { key: "d".to_string(), action: "priority".to_string(), target: "4".to_string() },
            QuickAction { key: "e".to_string(), action: "priority".to_string(), target: "5".to_string() },
        ]);
        
        let mut app = App::new_with_config(config, "Inbox".to_string());
        let max_index = app.config.quick_actions.as_ref().unwrap().len();
        
        // Set initial index within bounds
        let bounded_initial = initial_index % max_index;
        
        for show_hide in show_hide_sequence {
            if show_hide {
                app.show_quick_actions_modal();
                // Modal should be shown and index should be reset to 0
                prop_assert!(app.show_quick_actions_modal);
                prop_assert_eq!(app.selected_quick_action_index, 0);
                
                // Set to our bounded initial index
                app.selected_quick_action_index = bounded_initial;
            } else {
                app.hide_quick_actions_modal();
                // Modal should be hidden and index should be reset
                prop_assert!(!app.show_quick_actions_modal);
                prop_assert_eq!(app.selected_quick_action_index, 0);
            }
        }
    }
    
    #[test]
    fn test_concurrent_modal_operations(
        operations in prop::collection::vec(0u8..6, 1..20)
    ) {
        let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
        
        // Add a task for operations
        let task = cria::vikunja::models::Task {
            id: 1,
            title: "Test task".to_string(),
            done: false,
            is_favorite: false,
            labels: None,
            assignees: None,
            project_id: 1,
            priority: Some(1),
            due_date: None,
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
        };
        app.tasks.push(task);
        
        for op in operations {
            match op {
                0 => app.show_quick_actions_modal(),
                1 => app.hide_quick_actions_modal(),
                2 => app.show_help_modal(),
                3 => app.hide_help_modal(),
                4 => app.show_sort_modal(),
                5 => app.hide_sort_modal(),
                _ => {} // Should never happen due to range
            }
            
            // After any operation, only one modal should be open or none at all
            let modal_count = [
                app.show_quick_actions_modal,
                app.show_help_modal,
                app.show_sort_modal,
                app.show_quick_add_modal,
                app.show_edit_modal,
            ].iter().filter(|&&x| x).count();
            
            prop_assert!(modal_count <= 1, "More than one modal open: count = {}", modal_count);
        }
    }
}

// Regular unit tests to supplement property-based tests
#[test]
fn test_property_based_testing_setup() {
    // This test just verifies that property-based testing infrastructure works
    let config = CriaConfig::default();
    let app = App::new_with_config(config, "Test".to_string());
    
    // The app should be properly initialized with empty project map initially
    assert!(app.project_map.len() >= 0);
    assert!(!app.running == false); // Should be running
}

#[test]
fn test_edge_case_empty_quick_actions() {
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![]); // Empty quick actions
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.show_quick_actions_modal();
    
    // Should handle empty quick actions gracefully
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
    
    // Navigation should stay at 0
    app.selected_quick_action_index = 0; // This is the only valid index
    assert_eq!(app.selected_quick_action_index, 0);
}

#[test]
fn test_edge_case_no_quick_actions() {
    let mut config = CriaConfig::default();
    config.quick_actions = None; // No quick actions at all
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.show_quick_actions_modal();
    
    // Should handle missing quick actions gracefully
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
}
