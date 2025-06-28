use crate::vikunja::models::Task;

use super::state::{App, TaskFilter};

impl App {
    pub fn cycle_task_filter(&mut self) {
        self.task_filter = match self.task_filter {
            TaskFilter::ActiveOnly => TaskFilter::All,
            TaskFilter::All => TaskFilter::CompletedOnly,
            TaskFilter::CompletedOnly => TaskFilter::ActiveOnly,
        };
        self.apply_task_filter();
        self.selected_task_index = 0;
        let filter_name = match self.task_filter {
            TaskFilter::ActiveOnly => "Active Tasks Only",
            TaskFilter::All => "All Tasks",
            TaskFilter::CompletedOnly => "Completed Tasks Only",
        };
        self.add_debug_message(format!("Switched to filter: {}", filter_name));
    }

    pub fn apply_task_filter(&mut self) {
        self.tasks = match self.task_filter {
            TaskFilter::ActiveOnly => {
                self.all_tasks.iter().filter(|task| !task.done).cloned().collect()
            }
            TaskFilter::All => self.all_tasks.clone(),
            TaskFilter::CompletedOnly => {
                self.all_tasks.iter().filter(|task| task.done).cloned().collect()
            }
        };
        if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_task_index = self.tasks.len() - 1;
        }
    }

    pub fn get_filter_display_name(&self) -> &str {
        match self.task_filter {
            TaskFilter::ActiveOnly => "Active Only",
            TaskFilter::All => "All Tasks",
            TaskFilter::CompletedOnly => "Completed Only",
        }
    }

    pub fn should_hide_completed_immediately(&self) -> bool {
        matches!(self.task_filter, TaskFilter::ActiveOnly)
    }

    pub fn hide_filter_picker(&mut self) {
        self.show_filter_picker = false;
        self.filter_picker_input.clear();
    }
    pub fn add_char_to_filter_picker(&mut self, c: char) {
        self.filter_picker_input.push(c);
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }
    pub fn delete_char_from_filter_picker(&mut self) {
        self.filter_picker_input.pop();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }
    pub fn move_filter_picker_up(&mut self) {
        if self.selected_filter_picker_index > 0 {
            self.selected_filter_picker_index -= 1;
        }
    }
    pub fn move_filter_picker_down(&mut self) {
        if self.selected_filter_picker_index + 1 < self.filtered_filters.len() {
            self.selected_filter_picker_index += 1;
        }
    }
    pub fn select_filter_picker(&mut self) {
        if let Some((id, _)) = self.filtered_filters.get(self.selected_filter_picker_index) {
            if *id == -1 {
                self.current_filter_id = None;
            } else {
                self.current_filter_id = Some(*id);
            }
            self.apply_filter();
            self.hide_filter_picker();
        }
    }
    pub fn update_filtered_filters(&mut self) {
        let query = crate::tui::utils::normalize_string(&self.filter_picker_input);
        let mut filters: Vec<_> = self.filters.iter()
            .map(|(id, title)| (*id, title.clone()))
            .collect();
        if !query.is_empty() {
            filters.retain(|(_, title)| crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(title)) > 0);
            filters.sort_by(|a, b| {
                let sa = crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(&a.1));
                let sb = crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(&b.1));
                sb.cmp(&sa).then_with(|| crate::tui::utils::normalize_string(&a.1).cmp(&crate::tui::utils::normalize_string(&b.1)))
            });
        } else {
            filters.sort_by(|a, b| crate::tui::utils::normalize_string(&a.1).cmp(&crate::tui::utils::normalize_string(&b.1)));
        }
        self.filtered_filters = filters;
    }
    pub fn apply_filter(&mut self) {
        if let Some(fid) = self.current_filter_id {
            self.add_debug_message(format!("Applied filter ID: {}", fid));
        } else {
            self.apply_task_filter();
        }
        self.selected_task_index = 0;
    }
    pub fn apply_filter_tasks(&mut self, tasks: Vec<Task>) {
        self.all_tasks = tasks;
        self.apply_task_filter();
        self.selected_task_index = 0;
    }
}
