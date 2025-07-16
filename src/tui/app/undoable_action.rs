use crate::vikunja::models::Task;

#[derive(Clone, Debug)]
pub enum UndoableAction {
    #[allow(dead_code)]
    TaskCompletion {
        task_id: i64,
        previous_state: bool,
    },
    TaskDeletion {
        task: Task,
        position: usize,
    },
    TaskCreation {
        task_id: i64,
    },
    TaskEdit {
        task_id: i64,
        previous_task: Task,
    },
}
