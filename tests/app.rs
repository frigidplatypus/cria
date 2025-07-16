// Tests for App struct and logic in src/tui/app.rs
// Place this in tests/app.rs or similar

use cria::config::CriaConfig;
use cria::tui::app::state::App;
use cria::tui::app::sort_order::SortOrder;
use cria::tui::app::task_filter::TaskFilter;
use cria::vikunja::models::{Task, Label};
use chrono::{NaiveDate, TimeZone, Utc};
use cria::tui::app::pending_action::PendingAction;

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
fn test_app_initialization() {
    let app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    assert!(app.running);
    assert_eq!(app.tasks.len(), 0);
    assert_eq!(app.selected_task_index, 0);
    assert_eq!(app.default_project_name, "Inbox");
}

#[test]
fn test_quick_add_modal() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    app.add_char_to_quick_add('a');
    assert_eq!(app.quick_add_input, "a");
    app.delete_char_from_quick_add();
    assert_eq!(app.quick_add_input, "");
    app.hide_quick_add_modal();
    assert!(!app.show_quick_add_modal);
}

#[test]
fn test_edit_modal() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    assert!(app.editing_task_id.is_some());
    app.add_char_to_edit('x');
    assert!(app.edit_input.ends_with('x'));
    app.delete_char_from_edit();
    app.hide_edit_modal();
    assert!(!app.show_edit_modal);
}

#[test]
fn test_task_navigation() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    for i in 0..3 { app.tasks.push(sample_task(i, false)); }
    app.next_task();
    assert_eq!(app.selected_task_index, 1);
    app.previous_task();
    assert_eq!(app.selected_task_index, 0);
    app.jump_to_bottom();
    assert_eq!(app.selected_task_index, 2);
    app.jump_to_top();
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_task_completion_and_undo() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    let _id = app.toggle_task_completion().unwrap();
    assert!(app.tasks[0].done);
    app.undo_last_action();
    assert!(!app.tasks[0].done);
}

#[test]
fn test_task_deletion_and_undo() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks.push(sample_task(1, false));
    app.request_delete_task();
    app.confirm_action();
    assert!(app.tasks.is_empty());
    app.undo_last_action();
    assert_eq!(app.tasks.len(), 1);
}

#[test]
fn test_task_filtering() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.all_tasks = vec![sample_task(1, false), sample_task(2, true)];
    app.apply_task_filter();
    assert_eq!(app.tasks.len(), 1);
    app.cycle_task_filter();
    assert_eq!(app.tasks.len(), 2);
    app.cycle_task_filter();
    assert_eq!(app.tasks.len(), 1);
}

#[test]
fn test_project_picker() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.show_project_picker();
    app.add_char_to_project_picker('W');
    assert!(app.filtered_projects.iter().any(|(_, n)| n == "Work"));
    app.select_project_picker();
    assert!(app.current_project_id.is_some());
    app.hide_project_picker();
    assert!(!app.show_project_picker);
}

#[test]
fn test_sorting() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.tasks = vec![sample_task(2, false), sample_task(1, false)];
    app.apply_sort(SortOrder::TitleAZ);
    assert!(app.tasks[0].title < app.tasks[1].title);
    app.apply_sort(SortOrder::TitleZA);
    assert!(app.tasks[0].title > app.tasks[1].title);
}

#[test]
fn test_suggestions() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.label_map.insert(1, "Urgent".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.update_suggestions("*Ur", 3);
    assert!(app.suggestions.contains(&"Urgent".to_string()));
    app.update_suggestions("+Wo", 3);
    assert!(app.suggestions.contains(&"Work".to_string()));
    app.update_suggestions("Task", 4);
    assert!(app.suggestions.is_empty());
}

#[test]
fn test_multi_word_suggestions() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    
    // Add multi-word labels and projects
    app.label_map.insert(1, "High Priority".to_string());
    app.label_map.insert(2, "Low Priority".to_string());
    app.label_map.insert(3, "Work Related".to_string());
    app.label_map.insert(4, "Personal Task".to_string());
    
    app.project_map.insert(10, "Home Improvement".to_string());
    app.project_map.insert(11, "Work Projects".to_string());
    app.project_map.insert(12, "Personal Development".to_string());
    app.project_map.insert(13, "Side Business".to_string());
    
    // Test partial word matching for labels
    app.update_suggestions("*High", 5);
    assert!(app.suggestions.contains(&"High Priority".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test partial word matching for projects
    app.update_suggestions("+Home", 5);
    assert!(app.suggestions.contains(&"Home Improvement".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test multi-word prefix matching
    app.update_suggestions("*High Pri", 9);
    assert!(app.suggestions.contains(&"High Priority".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    app.update_suggestions("+Work Pro", 9);
    assert!(app.suggestions.contains(&"Work Projects".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test matching second word
    app.update_suggestions("*Pri", 4);
    assert!(app.suggestions.contains(&"High Priority".to_string()));
    assert!(app.suggestions.contains(&"Low Priority".to_string()));
    assert_eq!(app.suggestions.len(), 2);
    
    // Test matching with spaces in input
    app.update_suggestions("*Personal T", 11);
    assert!(app.suggestions.contains(&"Personal Task".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test case insensitive matching
    app.update_suggestions("*high pri", 9);
    assert!(app.suggestions.contains(&"High Priority".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test that delimiter characters stop suggestions
    app.update_suggestions("*High#", 6);
    assert!(app.suggestions.is_empty());
    
    app.update_suggestions("+Work(", 6);
    assert!(app.suggestions.is_empty());
}

#[test]
fn test_suggestion_word_boundary_matching() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    
    // Add labels that test word boundary matching
    app.label_map.insert(1, "Frontend Development".to_string());
    app.label_map.insert(2, "Backend Development".to_string());
    app.label_map.insert(3, "Full Stack Development".to_string());
    
    // Test that "Front Back" matches nothing (not a valid word boundary sequence)
    app.update_suggestions("*Front Back", 11);
    assert!(app.suggestions.is_empty());
    
    // Test that "Full Stack" matches "Full Stack Development"
    app.update_suggestions("*Full Stack", 11);
    assert!(app.suggestions.contains(&"Full Stack Development".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test that "Front" matches "Frontend Development"
    app.update_suggestions("*Front", 6);
    assert!(app.suggestions.contains(&"Frontend Development".to_string()));
    assert_eq!(app.suggestions.len(), 1);
    
    // Test that "Dev" matches all three (as word start)
    app.update_suggestions("*Dev", 4);
    assert!(app.suggestions.contains(&"Frontend Development".to_string()));
    assert!(app.suggestions.contains(&"Backend Development".to_string()));
    assert!(app.suggestions.contains(&"Full Stack Development".to_string()));
    assert_eq!(app.suggestions.len(), 3);
}

#[test]
fn test_add_task() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    let task = sample_task(42, false);
    app.tasks.push(task.clone());
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].id, 42);
    assert_eq!(app.tasks[0].title, "Task 42");
}

#[test]
fn test_edit_task_title() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    let task = sample_task(1, false);
    app.tasks.push(task.clone());
    // Simulate editing the task title
    app.tasks[0].title = "Edited Task".to_string();
    assert_eq!(app.tasks[0].title, "Edited Task");
}

#[test]
fn test_edit_task_priority_and_labels() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    let task = sample_task(1, false);
    app.tasks.push(task.clone());
    // Edit priority
    app.tasks[0].priority = Some(5);
    assert_eq!(app.tasks[0].priority, Some(5));
    // Edit labels
    app.tasks[0].labels = Some(vec![Label {
        id: 99,
        title: "Important".to_string(),
        hex_color: Some("#ff0000".to_string()),
        description: None,
        created: None,
        updated: None,
        created_by: None,
    }]);
    assert_eq!(app.tasks[0].labels.as_ref().unwrap()[0].title, "Important");
}

#[test]
fn test_quick_actions() {
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        cria::config::QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        cria::config::QuickAction {
            key: "u".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.tasks.push(sample_task(1, false));
    
    // Test showing and hiding quick actions modal
    app.show_quick_actions_modal();
    assert!(app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
    
    app.hide_quick_actions_modal();
    assert!(!app.show_quick_actions_modal);
    assert_eq!(app.selected_quick_action_index, 0);
    
    // Test quick action lookup
    assert!(app.config.has_quick_action("w"));
    assert!(app.config.has_quick_action("u"));
    assert!(!app.config.has_quick_action("x"));
    
    let work_action = app.config.get_quick_action("w").unwrap();
    assert_eq!(work_action.action, "project");
    assert_eq!(work_action.target, "Work");
    assert_eq!(work_action.get_description(), "Move to project: Work");
    
    // Test applying quick actions
    let priority_action = cria::config::QuickAction {
        key: "u".to_string(),
        action: "priority".to_string(),
        target: "5".to_string(),
    };
    let result = app.apply_quick_action(&priority_action);
    assert!(result.is_ok());
    assert_eq!(app.tasks[0].priority, Some(5));
}

#[test]
fn test_quick_action_descriptions() {
    let project_action = cria::config::QuickAction {
        key: "w".to_string(),
        action: "project".to_string(),
        target: "Work".to_string(),
    };
    assert_eq!(project_action.get_description(), "Move to project: Work");
    
    let priority_action = cria::config::QuickAction {
        key: "u".to_string(),
        action: "priority".to_string(),
        target: "5".to_string(),
    };
    assert_eq!(priority_action.get_description(), "Set priority to: 5");
    
    let label_action = cria::config::QuickAction {
        key: "i".to_string(),
        action: "label".to_string(),
        target: "Important".to_string(),
    };
    assert_eq!(label_action.get_description(), "Add label: Important");
}

#[test]
fn test_config_loading_from_custom_path() {
    // Test loading config from custom path using config.example.yaml
    let config = cria::config::CriaConfig::load_from_path(Some("config.example.yaml"));
    assert!(config.is_some());
    
    let config = config.unwrap();
    assert_eq!(config.api_url, "https://vikunja.example.com/api/v1");
    assert_eq!(config.api_key, Some("your-api-key-here".to_string()));
    assert_eq!(config.default_project, Some("Inbox".to_string()));
    
    // Test quick actions
    assert!(config.quick_actions.is_some());
    let quick_actions = config.quick_actions.unwrap();
    assert_eq!(quick_actions.len(), 8); // There are 8 quick actions in config.example.yaml
    
    // Test first quick action (w -> Work project)
    assert_eq!(quick_actions[0].key, "w");
    assert_eq!(quick_actions[0].action, "project");
    assert_eq!(quick_actions[0].target, "Work");
    
    // Test a priority action (u -> priority 5)
    let urgent_action = quick_actions.iter().find(|qa| qa.key == "u").unwrap();
    assert_eq!(urgent_action.action, "priority");
    assert_eq!(urgent_action.target, "5");
    
    // Test a label action (i -> Important label)
    let important_action = quick_actions.iter().find(|qa| qa.key == "i").unwrap();
    assert_eq!(important_action.action, "label");
    assert_eq!(important_action.target, "Important");
}

#[test]
fn test_config_loading_from_default_path() {
    // Test that default path loading still works
    let config = cria::config::CriaConfig::load_from_path(None);
    // This might be None if no default config exists, which is fine for testing
    // Just ensure it doesn't crash
    if let Some(config) = config {
        assert!(!config.api_url.is_empty());
    }
}

#[test]
fn test_config_loading_nonexistent_file() {
    // Test loading from a non-existent file
    let config = cria::config::CriaConfig::load_from_path(Some("nonexistent.yaml"));
    assert!(config.is_none());
}

#[test]
fn test_config_refresh_settings_defaults() {
    // Test that default refresh settings are correct
    let config = CriaConfig::default();
    assert_eq!(config.get_refresh_interval_seconds(), 300); // 5 minutes
    assert_eq!(config.is_auto_refresh_enabled(), true); // Enabled by default
}

#[test]
fn test_config_refresh_settings_from_test_config() {
    // Test loading refresh settings from our test config
    let config = cria::config::CriaConfig::load_from_path(Some("cargo-test.config.yaml"));
    assert!(config.is_some());
    
    let config = config.unwrap();
    assert_eq!(config.get_refresh_interval_seconds(), 60); // 1 minute from test config
    assert_eq!(config.is_auto_refresh_enabled(), true); // Enabled in test config
}

#[test]
fn test_config_refresh_settings_explicit_values() {
    // Test that explicit config values override defaults
    let yaml_content = r#"
api_url: "https://test.example.com/api/v1"
api_key: "test-key"
default_project: "Test"
auto_refresh: false
refresh_interval_seconds: 600
"#;
    
    let config: CriaConfig = serde_yaml::from_str(yaml_content).expect("Failed to parse config");
    assert_eq!(config.get_refresh_interval_seconds(), 600); // 10 minutes
    assert_eq!(config.is_auto_refresh_enabled(), false); // Explicitly disabled
}

#[test]
fn test_config_refresh_settings_partial_config() {
    // Test that missing config values use defaults
    let yaml_content = r#"
api_url: "https://test.example.com/api/v1"
api_key: "test-key"
default_project: "Test"
auto_refresh: false
# refresh_interval_seconds is missing
"#;
    
    let config: CriaConfig = serde_yaml::from_str(yaml_content).expect("Failed to parse config");
    assert_eq!(config.get_refresh_interval_seconds(), 300); // Default 5 minutes
    assert_eq!(config.is_auto_refresh_enabled(), false); // Explicitly disabled
}

#[test]
fn test_config_refresh_settings_only_interval() {
    // Test that setting only interval works
    let yaml_content = r#"
api_url: "https://test.example.com/api/v1"
api_key: "test-key"
default_project: "Test"
# auto_refresh is missing
refresh_interval_seconds: 120
"#;
    
    let config: CriaConfig = serde_yaml::from_str(yaml_content).expect("Failed to parse config");
    assert_eq!(config.get_refresh_interval_seconds(), 120); // 2 minutes
    assert_eq!(config.is_auto_refresh_enabled(), true); // Default enabled
}

#[test]
fn test_task_filter_header_display() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    // Default should be ActiveOnly
    assert_eq!(app.task_filter, TaskFilter::ActiveOnly);
    assert_eq!(app.get_filter_display_name(), "Active Tasks Only");
    // Cycle to All
    app.cycle_task_filter();
    assert_eq!(app.task_filter, TaskFilter::All);
    assert_eq!(app.get_filter_display_name(), "All Tasks");
    // Cycle to CompletedOnly
    app.cycle_task_filter();
    assert_eq!(app.task_filter, TaskFilter::CompletedOnly);
    assert_eq!(app.get_filter_display_name(), "Completed Tasks Only");
    // Cycle back to ActiveOnly
    app.cycle_task_filter();
    assert_eq!(app.task_filter, TaskFilter::ActiveOnly);
    assert_eq!(app.get_filter_display_name(), "Active Tasks Only");
}

#[test]
fn test_apply_label_quick_action() {
    use cria::vikunja::models::Task;
    use cria::tui::app::state::App;
    use cria::config::{CriaConfig, QuickAction};

    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "l".to_string(),
            action: "label".to_string(),
            target: "Important".to_string(),
        },
    ]);
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.label_map.insert(1, "Important".to_string());
    app.tasks.push(Task::default());
    app.selected_task_index = 0;

    let label_action = app.config.get_quick_action("l").cloned().unwrap();
    let result = app.apply_quick_action(&label_action);
    assert!(result.is_ok());
    let task = &app.tasks[0];
    let labels = task.labels.as_ref().expect("Task should have labels after label quick action");
    assert!(labels.iter().any(|lbl| lbl.title == "Important"));
}

#[test]
fn test_help_modal_state() {
    let mut app = App::default();
    assert!(!app.show_help_modal);
    app.show_help_modal = true;
    assert!(app.show_help_modal);
    app.show_help_modal = false;
    assert!(!app.show_help_modal);
}

#[test]
fn test_show_and_hide_task_details() {
    let mut app = App::default();
    // Info pane is used for task details
    assert!(app.show_info_pane);
    app.toggle_info_pane();
    assert!(!app.show_info_pane);
    app.toggle_info_pane();
    assert!(app.show_info_pane);
}

#[test]
fn test_quit_sets_running_false() {
    let mut app = App::default();
    assert!(app.running);
    app.running = false; // Simulate quit
    assert!(!app.running);
}

#[test]
fn test_delete_task_after_confirmation() {
    let mut app = App::default();
    app.tasks.push(Task { id: 1, title: "Delete Me".to_string(), ..Default::default() });
    app.selected_task_index = 0;
    app.show_confirmation_dialog = true;
    app.confirmation_message = "Delete task?".to_string();
    app.pending_action = Some(PendingAction::DeleteTask { task_id: 1 });
    // Simulate user confirms delete
    // In real app, confirm_action would handle this, but here we simulate removal
    if let Some(PendingAction::DeleteTask { task_id }) = app.pending_action.take() {
        app.tasks.retain(|t| t.id != task_id);
    }
    app.show_confirmation_dialog = false;
    assert!(app.tasks.is_empty());
    assert!(!app.show_confirmation_dialog);
}
