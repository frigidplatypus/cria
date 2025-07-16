use cria::tui::app::state::App;
use cria::vikunja::models::Task;

fn mock_task(id: i64, title: &str) -> Task {
    Task {
        id,
        title: title.to_string(),
        description: Some("desc".to_string()),
        ..Default::default()
    }
}

#[test]
fn test_show_edit_and_apply_edit_modal() {
    let mut app = App::default();
    // Add a task and select it
    app.tasks.push(mock_task(42, "Edit Me"));
    app.selected_task_index = 0;
    assert!(!app.show_edit_modal);
    app.show_edit_modal();
    assert!(app.show_edit_modal);
    assert_eq!(app.editing_task_id, Some(42));
    assert!(app.edit_input.contains("Edit Me"));
    // Simulate editing
    app.edit_input = "Changed Title".to_string();
    app.apply_edit_modal();
    // Assert the task was updated
    assert_eq!(app.tasks[0].title, "Changed Title");
    // Close and check state reset
    app.hide_edit_modal();
    assert!(!app.show_edit_modal);
    assert_eq!(app.edit_input, "");
    assert_eq!(app.edit_cursor_position, 0);
    assert_eq!(app.editing_task_id, None);
}
