use crate::vikunja::models::Task;

#[derive(Clone, Debug)]
pub struct FormEditState {
    pub field_index: usize,
    pub title: String,
    pub description: String,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub priority: Option<i32>,
    pub project_id: i64,
    pub label_ids: Vec<i64>,
    pub assignee_ids: Vec<i64>,
    pub is_favorite: bool,
    pub task_id: i64,
    pub comment: String,
    pub cursor_position: usize,
    // Removed unused fields: show_project_picker, show_label_picker
}

impl FormEditState {
    pub fn new(task: &Task) -> Self {
        Self {
            field_index: 0,
            title: task.title.clone(),
            description: task.description.clone().unwrap_or_default(),
            due_date: task.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
            start_date: task.start_date.map(|d| d.format("%Y-%m-%d").to_string()),
            priority: task.priority,
            project_id: task.project_id,
            label_ids: task.labels.as_ref().map(|labels| labels.iter().map(|l| l.id).collect()).unwrap_or_default(),
            assignee_ids: task.assignees.as_ref().map(|assignees| assignees.iter().map(|a| a.id).collect()).unwrap_or_default(),
            is_favorite: task.is_favorite,
            task_id: task.id,
            comment: String::new(),
            cursor_position: 0,
            // Removed unused fields: show_project_picker, show_label_picker
        }
    }
    pub fn get_field_count() -> usize {
        10
    }
    pub fn get_current_field_text(&self) -> String {
        match self.field_index {
            0 => self.title.clone(),
            1 => self.description.clone(),
            2 => self.due_date.clone().unwrap_or_default(),
            3 => self.start_date.clone().unwrap_or_default(),
            4 => self.priority.map(|p| p.to_string()).unwrap_or_default(),
            9 => self.comment.clone(),
            _ => String::new(),
        }
    }
    // Removed unused method: set_current_field_text
}
