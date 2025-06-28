use super::state::App;

impl App {
    pub fn hide_project_picker(&mut self) {
        self.show_project_picker = false;
        self.project_picker_input.clear();
    }
    pub fn add_char_to_project_picker(&mut self, c: char) {
        self.project_picker_input.push(c);
        self.update_filtered_projects();
        self.selected_project_picker_index = 0;
    }
    pub fn delete_char_from_project_picker(&mut self) {
        self.project_picker_input.pop();
        self.update_filtered_projects();
        self.selected_project_picker_index = 0;
    }
    pub fn move_project_picker_up(&mut self) {
        if self.selected_project_picker_index > 0 {
            self.selected_project_picker_index -= 1;
        }
    }
    pub fn move_project_picker_down(&mut self) {
        if self.selected_project_picker_index + 1 < self.filtered_projects.len() {
            self.selected_project_picker_index += 1;
        }
    }
    pub fn select_project_picker(&mut self) {
        if let Some((id, _)) = self.filtered_projects.get(self.selected_project_picker_index) {
            if *id == -1 {
                self.current_project_id = None;
            } else {
                self.current_project_id = Some(*id);
            }
            self.apply_project_filter();
            self.hide_project_picker();
        }
    }

    pub fn update_filtered_projects(&mut self) {
        let query = crate::tui::utils::normalize_string(&self.project_picker_input);
        let mut projects: Vec<_> = self.project_map.iter()
            .filter(|(id, _)| **id > 0)
            .map(|(id, name)| (*id, name.clone()))
            .collect();
        if !query.is_empty() {
            projects.retain(|(_, name)| crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(name)) > 0);
            projects.sort_by(|a, b| {
                let sa = crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(&a.1));
                let sb = crate::tui::utils::fuzzy_match_score(&query, &crate::tui::utils::normalize_string(&b.1));
                sb.cmp(&sa).then_with(|| crate::tui::utils::normalize_string(&a.1).cmp(&crate::tui::utils::normalize_string(&b.1)))
            });
        } else {
            projects.sort_by(|a, b| crate::tui::utils::normalize_string(&a.1).cmp(&crate::tui::utils::normalize_string(&b.1)));
        }
        if query.is_empty() {
            self.filtered_projects = vec![(-1, "All Projects".to_string())];
            self.filtered_projects.extend(projects);
        } else {
            self.filtered_projects = projects;
        }
    }

    pub fn apply_project_filter(&mut self) {
        if let Some(pid) = self.current_project_id {
            self.tasks = self.all_tasks.iter()
                .filter(|t| t.project_id == pid)
                .filter(|t| match self.task_filter {
                    super::state::TaskFilter::ActiveOnly => !t.done,
                    super::state::TaskFilter::All => true,
                    super::state::TaskFilter::CompletedOnly => t.done,
                })
                .cloned()
                .collect();
        } else {
            self.apply_task_filter();
        }
        self.selected_task_index = 0;
    }

    pub fn get_current_project_name(&self) -> String {
        if let Some(pid) = self.current_project_id {
            self.project_map.get(&pid).cloned().unwrap_or_else(|| "Unknown Project".to_string())
        } else {
            "All Projects".to_string()
        }
    }
}
