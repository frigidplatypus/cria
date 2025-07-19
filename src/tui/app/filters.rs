use crate::tui::app::state::App;
use crate::tui::utils::contains_ignore_case;

impl App {
    #[allow(dead_code)]
    pub fn show_filter_picker(&mut self) {
        self.close_all_modals();
        self.show_filter_picker = true;
        self.filter_picker_input.clear();
        self.selected_filter_picker_index = 0;
        self.update_filtered_filters();
    }
    pub fn hide_filter_picker(&mut self) {
        self.show_filter_picker = false;
        self.filter_picker_input.clear();
    }
    #[allow(dead_code)]
    pub fn add_char_to_filter_picker(&mut self, c: char) {
        self.filter_picker_input.insert(self.selected_filter_picker_index, c);
        self.selected_filter_picker_index += 1;
        self.update_filtered_filters();
    }
    #[allow(dead_code)]
    pub fn delete_char_from_filter_picker(&mut self) {
        if self.selected_filter_picker_index > 0 {
            self.selected_filter_picker_index -= 1;
            self.filter_picker_input.remove(self.selected_filter_picker_index);
            self.update_filtered_filters();
        }
    }
    #[allow(dead_code)]
    pub fn move_filter_picker_up(&mut self) {
        if !self.filtered_filters.is_empty() {
            self.selected_filter_picker_index = (self.selected_filter_picker_index + self.filtered_filters.len() - 1) % self.filtered_filters.len();
        }
    }
    #[allow(dead_code)]
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
        
        // Add "Clear Filter" option if a filter is currently active
        if self.current_filter_id.is_some() {
            self.filtered_filters.insert(0, (-1, "Clear Filter".to_string()));
        }
    }
    pub fn set_filters(&mut self, filters: Vec<(i64, String, Option<String>)>) {
        self.filters = filters.iter().map(|(id, title, _)| (*id, title.clone())).collect();
        self.filter_descriptions = filters.into_iter()
            .filter_map(|(id, _, desc)| desc.map(|d| (id, d)))
            .collect();
        self.update_filtered_filters();
    }
    #[allow(dead_code)]
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
            crate::tui::app::task_filter::TaskFilter::ActiveOnly => !task.done,
            crate::tui::app::task_filter::TaskFilter::All => true,
            crate::tui::app::task_filter::TaskFilter::CompletedOnly => task.done,
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
            // Show task filter state if no saved filter is selected
            match self.task_filter {
                crate::tui::app::task_filter::TaskFilter::ActiveOnly => "Active Tasks Only".to_string(),
                crate::tui::app::task_filter::TaskFilter::All => "All Tasks".to_string(),
                crate::tui::app::task_filter::TaskFilter::CompletedOnly => "Completed Tasks Only".to_string(),
            }
        }
    }
    pub fn cycle_task_filter(&mut self) {
        self.task_filter = match self.task_filter {
            crate::tui::app::task_filter::TaskFilter::ActiveOnly => crate::tui::app::task_filter::TaskFilter::All,
            crate::tui::app::task_filter::TaskFilter::All => crate::tui::app::task_filter::TaskFilter::CompletedOnly,
            crate::tui::app::task_filter::TaskFilter::CompletedOnly => crate::tui::app::task_filter::TaskFilter::ActiveOnly,
        };
        
        // If we're currently viewing a specific project, apply project filter (which includes task filter)
        // Otherwise, just apply the task filter to all tasks
        if self.current_project_id.is_some() {
            self.apply_project_filter();
        } else {
            self.apply_task_filter();
        }
    }
    pub fn update_all_tasks(&mut self, tasks: Vec<crate::vikunja::models::Task>) {
        self.all_tasks = tasks.clone();
        self.reapply_current_filters();
    }

    /// Reapply all current filters after data updates
    pub fn reapply_current_filters(&mut self) {
        if let Some(filter_id) = self.current_filter_id {
            // If a saved filter is active, we need to fetch tasks for that filter
            // This should be handled by the calling code that has access to the API client
            // For now, fall back to task filter
            self.apply_task_filter();
        } else if self.current_project_id.is_some() {
            // If a project is selected, apply project filter
            self.apply_project_filter();
        } else {
            // If no special filter is active, just apply the task filter
            self.apply_task_filter();
        }
    }
    /// Extract project override from filter description
    /// Looks for "cria_project: ProjectName" in description and returns the project name
    pub fn extract_project_override(&self, filter_id: i64) -> Option<String> {
        crate::debug::debug_log(&format!("extract_project_override: Checking filter_id={}", filter_id));
        
        if let Some(description) = self.filter_descriptions.get(&filter_id) {
            crate::debug::debug_log(&format!("extract_project_override: Found description: '{}'", description));
            
            // Look for pattern "cria_project: ProjectName"
            if let Some(start) = description.find("cria_project:") {
                let after_colon = &description[start + "cria_project:".len()..];
                // Find the project name - everything up to the next space, HTML tag, or end of line
                let mut project_name = after_colon.trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim();
                
                // Remove common HTML closing tags if present
                if let Some(tag_start) = project_name.find('<') {
                    project_name = &project_name[..tag_start];
                }
                
                project_name = project_name.trim();
                
                if !project_name.is_empty() {
                    crate::debug::debug_log(&format!("extract_project_override: Extracted project name: '{}'", project_name));
                    return Some(project_name.to_string());
                }
            }
            crate::debug::debug_log("extract_project_override: No 'cria_project:' pattern found in description");
        } else {
            crate::debug::debug_log(&format!("extract_project_override: No description found for filter_id={}", filter_id));
        }
        None
    }

    /// Apply filter and handle project override
    pub fn apply_filter_with_override(&mut self, filter_id: i64) {
        crate::debug::debug_log(&format!("apply_filter_with_override: Processing filter_id={}", filter_id));
        
        self.current_filter_id = Some(filter_id);
        
        // Check for project override in filter description
        if let Some(project_name) = self.extract_project_override(filter_id) {
            crate::debug::debug_log(&format!("apply_filter_with_override: Project override detected: '{}'", project_name));
            self.active_project_override = Some(project_name.clone());
            self.show_toast(format!("Default project overridden to: {}", project_name));
            crate::debug::debug_log(&format!("apply_filter_with_override: Toast shown for project override: '{}'", project_name));
        } else {
            crate::debug::debug_log("apply_filter_with_override: No project override found in filter description");
            self.active_project_override = None;
        }
        
        crate::debug::debug_log(&format!("apply_filter_with_override: Final state - filter_id={:?}, override={:?}", 
                                        self.current_filter_id, self.active_project_override));
    }

    /// Get the currently active default project name (considering override)
    pub fn get_active_default_project(&self) -> String {
        if let Some(ref override_project) = self.active_project_override {
            override_project.clone()
        } else {
            self.default_project_name.clone()
        }
    }

    /// Clear filter and reset project override
    pub fn clear_filter(&mut self) {
        self.current_filter_id = None;
        if self.active_project_override.is_some() {
            self.active_project_override = None;
            self.show_toast("Default project restored".to_string());
        }
    }

    /// Find filter by name
    pub fn find_filter_by_name(&self, name: &str) -> Option<i64> {
        self.filters.iter()
            .find(|(_, title)| title.eq_ignore_ascii_case(name))
            .map(|(id, _)| *id)
    }

    /// Apply default filter from config if specified
    pub async fn apply_default_filter_from_config(&mut self, config: &crate::config::CriaConfig, api_client: &std::sync::Arc<tokio::sync::Mutex<crate::vikunja_client::VikunjaClient>>) {
        if let Some(ref default_filter_name) = config.default_filter {
            crate::debug::debug_log(&format!("Attempting to apply default filter: '{}'", default_filter_name));
            
            if let Some(filter_id) = self.find_filter_by_name(default_filter_name) {
                crate::debug::debug_log(&format!("Found default filter '{}' with ID: {}", default_filter_name, filter_id));
                
                // Apply the filter with override (similar to filter picker logic)
                self.apply_filter_with_override(filter_id);
                
                // Fetch tasks for the filter
                match api_client.lock().await.get_tasks_for_filter(filter_id).await {
                    Ok(tasks) => {
                        crate::debug::debug_log(&format!("Default filter: Got {} tasks for filter '{}'", tasks.len(), default_filter_name));
                        self.apply_filter_tasks(tasks);
                        self.show_toast(format!("Applied default filter: {}", default_filter_name));
                    },
                    Err(e) => {
                        crate::debug::debug_log(&format!("Default filter: Failed to fetch tasks for filter '{}': {}", default_filter_name, e));
                        self.show_toast(format!("Failed to load default filter: {}", default_filter_name));
                    }
                }
            } else {
                crate::debug::debug_log(&format!("Default filter '{}' not found in available filters", default_filter_name));
                self.show_toast(format!("Default filter '{}' not found", default_filter_name));
            }
        }
    }
}
