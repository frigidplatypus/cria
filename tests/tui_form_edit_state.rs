use cria::tui::app::form_edit_state::FormEditState;
use cria::vikunja::models::Task;

fn mock_task_with_labels_and_project() -> Task {
    let mut task = Task::default();
    task.id = 1;
    task.title = "Test Task".to_string();
    task.project_id = 42;
    // Simulate labels with ids 10 and 20
    task.labels = Some(vec![
        cria::vikunja::models::Label {
            id: 10,
            title: "Label 10".to_string(),
            hex_color: None,
            description: None,
            created: None,
            updated: None,
            created_by: None,
        },
        cria::vikunja::models::Label {
            id: 20,
            title: "Label 20".to_string(),
            hex_color: None,
            description: None,
            created: None,
            updated: None,
            created_by: None,
        },
    ]);
    task
}

#[test]
fn test_set_project_id_in_form_edit_state() {
    let task = mock_task_with_labels_and_project();
    let mut form = FormEditState::new(&task);
    assert_eq!(form.project_id, 42);
    form.set_project_id(99);
    assert_eq!(form.project_id, 99);
}

#[test]
fn test_set_label_ids_in_form_edit_state() {
    let task = mock_task_with_labels_and_project();
    let mut form = FormEditState::new(&task);
    assert_eq!(form.label_ids, vec![10, 20]);
    form.set_label_ids(vec![30, 40, 50]);
    assert_eq!(form.label_ids, vec![30, 40, 50]);
}
