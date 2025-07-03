use crate::tui::app::App;
use crate::tui::utils::contains_ignore_case;

impl App {
    pub fn show_filter_picker(&mut self) {
        self.show_filter_picker = true;
        self.filter_picker_input.clear();
        self.selected_filter_picker_index = 0;
        self.update_filtered_filters();
    }
    pub fn hide_filter_picker(&mut self) {
        self.show_filter_picker = false;
        self.filter_picker_input.clear();
    }
    pub fn add_char_to_filter_picker(&mut self, c: char) {
        self.filter_picker_input.insert(self.selected_filter_picker_index, c);
        self.selected_filter_picker_index += 1;
        self.update_filtered_filters();
    }
    pub fn delete_char_from_filter_picker(&mut self) {
        if self.selected_filter_picker_index > 0 {
            self.selected_filter_picker_index -= 1;
            self.filter_picker_input.remove(self.selected_filter_picker_index);
            self.update_filtered_filters();
        }
    }
    pub fn move_filter_picker_up(&mut self) {
        if !self.filtered_filters.is_empty() {
            self.selected_filter_picker_index = (self.selected_filter_picker_index + self.filtered_filters.len() - 1) % self.filtered_filters.len();
        }
    }
    pub fn move_filter_picker_down(&mut self) {
        if !self.filtered_filters.is_empty() {
            self.selected_filter_picker_index = (self.selected_filter_picker_index + 1) % self.filtered_filters.len();
        }
    }
    #[allow(dead_code)] // Future feature
    pub fn select_filter_picker(&mut self) {
        if let Some(filter) = self.filtered_filters.get(self.selected_filter_picker_index) {
            self.current_filter_id = Some(filter.0);
            self.filter_picker_input = filter.1.clone();
            self.hide_filter_picker();
        }
    }
    pub fn update_filtered_filters(&mut self) {
        let query = &self.filter_picker_input;
        self.filtered_filters = self.filters.iter()
            .filter(|(_, title)| contains_ignore_case(title, query))
            .map(|(id, title)| (*id, title.clone()))
            .collect::<Vec<_>>();
    }
    pub fn set_filters(&mut self, filters: Vec<(i64, String)>) {
        self.filters = filters;
        self.update_filtered_filters();
    }
    pub fn apply_filter_tasks(&mut self, tasks: Vec<crate::vikunja::models::Task>) {
        self.tasks = tasks;
    }
    #[allow(dead_code)] // Future feature
    pub fn apply_filter(&mut self) {
        if let Some(_filter_id) = self.current_filter_id {
            // No filter_id on Task, so this is a placeholder for actual filter logic
            // self.tasks = self.all_tasks.iter().filter(|task| task.filter_id == filter_id).cloned().collect();
        }
    }
    #[allow(dead_code)] // Future feature
    pub fn get_current_filter_name(&self) -> String {
        if let Some(filter_id) = self.current_filter_id {
            if let Some(title) = self.filters.iter().find(|f| f.0 == filter_id).map(|f| &f.1) {
                return title.clone();
            }
        }
        "No filter".to_string()
    }
    pub fn apply_task_filter(&mut self) {
        self.tasks = self.all_tasks.iter().filter(|task| match self.task_filter {
            crate::tui::app::state::TaskFilter::ActiveOnly => !task.done,
            crate::tui::app::state::TaskFilter::All => true,
            crate::tui::app::state::TaskFilter::CompletedOnly => task.done,
        }).cloned().collect();
        
        // Apply layout-specific sort if no manual sort is active
        if self.current_sort.is_none() {
            self.apply_layout_sort();
        }
    }
    pub fn get_filter_display_name(&self) -> String {
        if let Some(filter_id) = self.current_filter_id {
            if let Some(filter) = self.filters.iter().find(|f| f.0 == filter_id) {
                return filter.1.clone();
            }
            format!("Filter {}", filter_id)
        } else {
            "All Tasks".to_string()
        }
    }
    pub fn cycle_task_filter(&mut self) {
        self.task_filter = match self.task_filter {
            crate::tui::app::state::TaskFilter::ActiveOnly => crate::tui::app::state::TaskFilter::All,
            crate::tui::app::state::TaskFilter::All => crate::tui::app::state::TaskFilter::CompletedOnly,
            crate::tui::app::state::TaskFilter::CompletedOnly => crate::tui::app::state::TaskFilter::ActiveOnly,
        };
        self.apply_task_filter();
    }
    pub fn update_all_tasks(&mut self, tasks: Vec<crate::vikunja::models::Task>) {
        self.all_tasks = tasks.clone();
        self.tasks = tasks.into_iter().filter(|task| match self.task_filter {
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
