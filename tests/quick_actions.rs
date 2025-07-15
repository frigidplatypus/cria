//! Integration tests for quick actions in the CRIA TUI app

use cria::tui::app::state::App;
use cria::config::{CriaConfig, QuickAction};
use cria::vikunja::models::Task;

#[tokio::test]
async fn test_quick_action_by_shortcut_key() {
    // Setup a minimal config with a quick action
    let quick_action = QuickAction {
        key: "w".to_string(),
        action: "project".to_string(),
        target: "Western".to_string(),
    };
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![quick_action.clone()]);

    // Create app with a dummy task
    let mut app = App::new_with_config(config, "Inbox".to_string());
    app.all_tasks.push(Task {
        id: 1,
        title: "Test Task".to_string(),
        project_id: 42,
        ..Default::default()
    });
    app.tasks = app.all_tasks.clone();
    app.selected_task_index = 0;
    // Add the target project to the project_map so the quick action can succeed
    app.project_map.insert(99, "Western".to_string());

    // Simulate triggering the quick action by shortcut key
    let result = app.apply_quick_action(&quick_action);
    assert!(result.is_ok(), "Quick action should apply without error, got: {:?}", result.err());
    // TODO: Add assertions for the expected effect of the quick action
}

#[tokio::test]
async fn test_quick_action_modal_enter() {
    // Similar to above, but simulate selection by index and Enter
    // ...
}
