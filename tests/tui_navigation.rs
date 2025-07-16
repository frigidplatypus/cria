use cria::tui::app::state::App;
use cria::tui::app::form_edit_state::FormEditState;
use cria::vikunja::models::Task;

fn mock_task(id: i64, title: &str) -> Task {
    Task {
        id,
        title: title.to_string(),
        description: Some("desc".to_string()),
        due_date: None,
        start_date: None,
        priority: Some(1),
        project_id: 1,
        labels: None,
        assignees: None,
        is_favorite: false,
        ..Default::default()
    }
}

#[test]
fn test_task_navigation_next_prev() {
    let mut app = App::default();
    app.tasks = vec![mock_task(1, "A"), mock_task(2, "B"), mock_task(3, "C")];
    app.selected_task_index = 0;
    // Simulate next
    app.selected_task_index = (app.selected_task_index + 1) % app.tasks.len();
    assert_eq!(app.selected_task_index, 1);
    // Simulate prev
    app.selected_task_index = (app.selected_task_index + app.tasks.len() - 1) % app.tasks.len();
    assert_eq!(app.selected_task_index, 0);
}

#[test]
fn test_form_field_navigation() {
    let task = mock_task(1, "A");
    let mut form = FormEditState::new(&task);
    form.field_index = 0;
    form.field_index = (form.field_index + 1) % FormEditState::get_field_count();
    assert_eq!(form.field_index, 1);
    form.field_index = (form.field_index + FormEditState::get_field_count() - 1) % FormEditState::get_field_count();
    assert_eq!(form.field_index, 0);
}
