use crate::tui::app::{App, PickerContext};
use crate::tui::utils::contains_ignore_case;

impl App {
    pub fn show_project_picker(&mut self) {
        self.close_all_modals();
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
        self.project_picker_input.push(c);
        self.update_filtered_projects();
        self.selected_project_picker_index = 0; // Reset selection to first item
    }
    pub fn delete_char_from_project_picker(&mut self) {
        if !self.project_picker_input.is_empty() {
            self.project_picker_input.pop();
            self.update_filtered_projects();
            self.selected_project_picker_index = 0; // Reset selection to first item
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
        if let Some((project_id, project_name)) = self.filtered_projects.get(self.selected_project_picker_index).cloned() {
            let picker_context = self.picker_context.clone();
            if picker_context == PickerContext::FormEditProject {
                // Only update the form, do not touch main task list/filter
                self.hide_project_picker();
                if let Some(ref mut form) = self.form_edit_state {
                    form.project_id = project_id;
                }
                self.show_form_edit_modal = true;
                self.picker_context = PickerContext::None;
                return;
            }
            // Normal (non-form) picker behavior
            if project_id == -1 {
                self.current_project_id = None;
                self.project_picker_input = project_name.clone();
                self.hide_project_picker();
                // Apply task filter to show all tasks respecting current filter (Active/All/Completed)
                self.apply_task_filter();
            } else {
                self.current_project_id = Some(project_id);
                self.project_picker_input = project_name.clone();
                self.hide_project_picker();
                self.apply_project_filter();
            }
        }
    }
    pub fn update_filtered_projects(&mut self) {
        let query = &self.project_picker_input;
        self.filtered_projects = self.project_map.iter()
            .filter(|(_, name)| contains_ignore_case(name, query))
            .map(|(id, name)| (*id, name.clone()))
            .collect::<Vec<_>>();
        // Add 'All Projects' option if a project is selected
        if self.current_project_id.is_some() {
            self.filtered_projects.insert(0, (-1, "All Projects".to_string()));
        }
    }
    #[allow(dead_code)] // Future feature
    pub fn apply_project_filter(&mut self) {
        if let Some(project_id) = self.current_project_id {
            // First filter by project, then apply task filter (Active/All/Completed)
            let project_tasks: Vec<_> = self.all_tasks.iter()
                .filter(|task| task.project_id == project_id)
                .cloned()
                .collect();
            
            // Apply task filter to the project-filtered tasks
            self.tasks = project_tasks.into_iter().filter(|task| match self.task_filter {
                crate::tui::app::state::TaskFilter::ActiveOnly => !task.done,
                crate::tui::app::state::TaskFilter::All => true,
                crate::tui::app::state::TaskFilter::CompletedOnly => task.done,
            }).collect();
            
            // Apply layout-specific sort if no manual sort is active
            if self.current_sort.is_none() {
                self.apply_layout_sort();
            }
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
