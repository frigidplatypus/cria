#[derive(Clone, Debug)]
pub enum PendingAction {
    DeleteTask { task_id: i64 },
    QuitApp,
}
