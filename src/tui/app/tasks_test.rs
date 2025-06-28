#[cfg(test)]
mod tests {
    use super::*;
    use crate::vikunja::models::Task;

    #[test]
    fn undo_task_completion_and_deletion() {
        let mut app = App::new();
        // Add a task
        let task = Task {
            id: 1,
            title: "Test Task".to_string(),
            done: false,
            ..Default::default()
        };
        app.tasks.push(task.clone());
        app.all_tasks.push(task.clone());
        app.selected_task_index = 0;
        // Complete the task
        app.toggle_task_completion();
        assert!(app.tasks[0].done);
        // Undo completion
        app.undo_last_action();
        assert!(!app.tasks[0].done);
        // Delete the task
        app.request_delete_task();
        app.confirm_action();
        assert!(app.tasks.is_empty());
        // Undo deletion
        app.undo_last_action();
        assert_eq!(app.tasks.len(), 1);
        assert_eq!(app.tasks[0].title, "Test Task");
    }
}
