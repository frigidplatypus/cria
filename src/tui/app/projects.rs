use crate::tui::app::App;
use crate::tui::utils::contains_ignore_case;

impl App {
    pub fn show_project_picker(&mut self) {
        self.show_project_picker = true;
        self.project_picker_input.clear();
        self.selected_project_picker_index = 0;
        self.update_filtered_projects();
    }
    pub fn hide_project_picker(&mut self) {
        self.show_project_picker = false;
        self.project_picker_input.clear();
    }
    pub fn add_char_to_project_picker(&mut self, c: char) {
        self.project_picker_input.insert(self.selected_project_picker_index, c);
        self.selected_project_picker_index += 1;
        self.update_filtered_projects();
    }
    pub fn delete_char_from_project_picker(&mut self) {
        if self.selected_project_picker_index > 0 {
            self.selected_project_picker_index -= 1;
            self.project_picker_input.remove(self.selected_project_picker_index);
            self.update_filtered_projects();
        }
    }
    pub fn move_project_picker_up(&mut self) {
        if !self.filtered_projects.is_empty() {
            self.selected_project_picker_index = (self.selected_project_picker_index + self.filtered_projects.len() - 1) % self.filtered_projects.len();
        }
    }
    pub fn move_project_picker_down(&mut self) {
        if !self.filtered_projects.is_empty() {
            self.selected_project_picker_index = (self.selected_project_picker_index + 1) % self.filtered_projects.len();
        }
    }
    pub fn select_project_picker(&mut self) {
        if let Some(project) = self.filtered_projects.get(self.selected_project_picker_index) {
            self.current_project_id = Some(project.0);
            self.project_picker_input = project.1.clone();
            self.hide_project_picker();
        }
    }
    pub fn update_filtered_projects(&mut self) {
        let query = &self.project_picker_input;
        self.filtered_projects = self.project_map.iter()
            .filter(|(_, name)| contains_ignore_case(name, query))
            .map(|(id, name)| (*id, name.clone()))
            .collect::<Vec<_>>();
    }
    pub fn apply_project_filter(&mut self) {
        if let Some(project_id) = self.current_project_id {
            self.tasks = self.all_tasks.iter().filter(|task| task.project_id == project_id).cloned().collect();
        }
    }
    pub fn get_current_project_name(&self) -> String {
        if let Some(project_id) = self.current_project_id {
            if let Some(name) = self.project_map.get(&project_id) {
                return name.clone();
            }
        }
        "No project".to_string()
    }
}
