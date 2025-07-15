#[cfg(test)]
mod tests {
    use super::*;
    use crate::vikunja::models::Task;

    fn mock_task() -> Task {
        Task {
            id: 1,
            title: "Test Task".to_string(),
            description: Some("Description".to_string()),
            due_date: None,
            start_date: None,
            priority: Some(2),
            project_id: 42,
            labels: None,
            assignees: None,
            is_favorite: false,
            // ...add other fields as needed with default/mock values
            ..Default::default()
        }
    }

    #[test]
    fn test_new_form_edit_state() {
        let task = mock_task();
        let form = FormEditState::new(&task);
        assert_eq!(form.title, "Test Task");
        assert_eq!(form.description, "Description");
        assert_eq!(form.project_id, 42);
        assert_eq!(form.priority, Some(2));
    }

    #[test]
    fn test_get_current_field_text() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 0;
        assert_eq!(form.get_current_field_text(), "Test Task");
        form.field_index = 1;
        assert_eq!(form.get_current_field_text(), "Description");
    }

    #[test]
    fn test_set_current_field_text_title() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 0;
        form.set_current_field_text("New Title".to_string());
        assert_eq!(form.title, "New Title");
    }

    #[test]
    fn test_quick_add_modal_integration() {
        // Simulate a quick add input string as a user would enter in the modal
        let input = "Buy groceries *shopping @john +personal tomorrow !2";
        // Use the parser directly (integration with modal logic)
        use crate::vikunja_parser::QuickAddParser;
        let parser = QuickAddParser::new();
        let parsed = parser.parse(input);

        // Check that all fields are parsed as expected
        assert_eq!(parsed.title, "Buy groceries");
        assert_eq!(parsed.labels, vec!["shopping"]);
        assert_eq!(parsed.assignees, vec!["john"]);
        assert_eq!(parsed.project, Some("personal".to_string()));
        assert_eq!(parsed.priority, Some(2));
        assert!(parsed.due_date.is_some());
    }

    #[test]
    fn test_form_edit_state_field_navigation_and_editing() {
        let task = mock_task();
        let mut form = FormEditState::new(&task);
        // There are 10 fields (see get_field_count)
        let field_names = [
            "title", "description", "due_date", "start_date", "priority", "", "", "", "", "comment"
        ];
        // Tab through all fields and set a value for each editable one
        let test_values = [
            "New Title", "New Description", "2025-12-31", "2025-11-01", "5", "", "", "", "", "A comment"
        ];
        for i in 0..FormEditState::get_field_count() {
            form.field_index = i;
            // Only set for fields that are editable
            if !test_values[i].is_empty() {
                form.set_current_field_text(test_values[i].to_string());
                let value = form.get_current_field_text();
                assert_eq!(value, test_values[i]);
            }
        }
        // Check that all fields were set correctly
        assert_eq!(form.title, "New Title");
        assert_eq!(form.description, "New Description");
        assert_eq!(form.due_date, Some("2025-12-31".to_string()));
        assert_eq!(form.start_date, Some("2025-11-01".to_string()));
        assert_eq!(form.priority, Some(5));
        assert_eq!(form.comment, "A comment");
    }

    #[test]
    fn test_form_edit_state_priority_parsing() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 4; // priority
        form.set_current_field_text("7".to_string());
        assert_eq!(form.priority, Some(7));
        form.set_current_field_text("notanumber".to_string());
        assert_eq!(form.priority, None); // Should fail to parse
    }

    #[test]
    fn test_form_edit_state_due_and_start_date_empty() {
        let mut form = FormEditState::new(&mock_task());
        form.field_index = 2; // due_date
        form.set_current_field_text("".to_string());
        assert_eq!(form.due_date, None);
        form.field_index = 3; // start_date
        form.set_current_field_text("".to_string());
        assert_eq!(form.start_date, None);
    }
}
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
    /// Set the project_id for the form editor
    pub fn set_project_id(&mut self, project_id: i64) {
        self.project_id = project_id;
    }

    /// Set the label_ids for the form editor
    pub fn set_label_ids(&mut self, label_ids: Vec<i64>) {
        self.label_ids = label_ids;
    }
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
