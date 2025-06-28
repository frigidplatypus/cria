use super::state::App;

impl App {
    pub fn hide_quick_add_modal(&mut self) {
        self.show_quick_add_modal = false;
        self.quick_add_input.clear();
        self.quick_add_cursor_position = 0;
    }
    pub fn add_char_to_quick_add(&mut self, c: char) {
        self.quick_add_input.insert(self.quick_add_cursor_position, c);
        self.quick_add_cursor_position += 1;
    }
    pub fn delete_char_from_quick_add(&mut self) {
        if self.quick_add_cursor_position > 0 {
            self.quick_add_cursor_position -= 1;
            self.quick_add_input.remove(self.quick_add_cursor_position);
        }
    }
    pub fn move_cursor_left(&mut self) {
        if self.quick_add_cursor_position > 0 {
            self.quick_add_cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.quick_add_cursor_position < self.quick_add_input.len() {
            self.quick_add_cursor_position += 1;
        }
    }
    pub fn hide_edit_modal(&mut self) {
        self.show_edit_modal = false;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.editing_task_id = None;
    }
    pub fn add_char_to_edit(&mut self, c: char) {
        self.edit_input.insert(self.edit_cursor_position, c);
        self.edit_cursor_position += 1;
    }
    pub fn delete_char_from_edit(&mut self) {
        if self.edit_cursor_position > 0 {
            self.edit_cursor_position -= 1;
            self.edit_input.remove(self.edit_cursor_position);
        }
    }
    pub fn move_edit_cursor_left(&mut self) {
        if self.edit_cursor_position > 0 {
            self.edit_cursor_position -= 1;
        }
    }
    pub fn move_edit_cursor_right(&mut self) {
        if self.edit_cursor_position < self.edit_input.len() {
            self.edit_cursor_position += 1;
        }
    }
    pub fn update_suggestions(&mut self, input: &str, cursor: usize) {
        let before_cursor = &input[..cursor];
        if let Some(pos) = before_cursor.rfind('*') {
            let after = &before_cursor[pos+1..];
            if after.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                self.suggestion_mode = Some(super::state::SuggestionMode::Label);
                self.suggestion_prefix = after.to_string();
                let prefix = crate::tui::utils::normalize_string(after);
                let mut labels: Vec<_> = self.label_map.values().cloned().collect();
                labels.sort_by(|a, b| crate::tui::utils::normalize_string(a).cmp(&crate::tui::utils::normalize_string(b)));
                let mut filtered: Vec<_> = labels.into_iter()
                    .filter(|l| crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(l)) > 0)
                    .collect();
                filtered.sort_by(|a, b| {
                    let sa = crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(a));
                    let sb = crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(b));
                    sb.cmp(&sa).then_with(|| crate::tui::utils::normalize_string(a).cmp(&crate::tui::utils::normalize_string(b)))
                });
                if filtered != self.suggestions {
                    self.selected_suggestion = 0;
                } else if self.selected_suggestion >= filtered.len() {
                    self.selected_suggestion = 0;
                }
                self.suggestions = filtered;
                return;
            }
        }
        if let Some(pos) = before_cursor.rfind('+') {
            let after = &before_cursor[pos+1..];
            if after.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                self.suggestion_mode = Some(super::state::SuggestionMode::Project);
                self.suggestion_prefix = after.to_string();
                let prefix = crate::tui::utils::normalize_string(after);
                let mut projects: Vec<_> = self.project_map.iter()
                    .filter(|(id, _)| **id > 0)
                    .map(|(_, name)| name.clone())
                    .collect();
                projects.sort_by(|a, b| crate::tui::utils::normalize_string(a).cmp(&crate::tui::utils::normalize_string(b)));
                let mut filtered: Vec<_> = projects.into_iter()
                    .filter(|p| crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(p)) > 0)
                    .collect();
                filtered.sort_by(|a, b| {
                    let sa = crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(a));
                    let sb = crate::tui::utils::fuzzy_match_score(&prefix, &crate::tui::utils::normalize_string(b));
                    sb.cmp(&sa).then_with(|| crate::tui::utils::normalize_string(a).cmp(&crate::tui::utils::normalize_string(b)))
                });
                if filtered != self.suggestions {
                    self.selected_suggestion = 0;
                } else if self.selected_suggestion >= filtered.len() {
                    self.selected_suggestion = 0;
                }
                self.suggestions = filtered;
                return;
            }
        }
        self.suggestion_mode = None;
        self.suggestions.clear();
        self.selected_suggestion = 0;
        self.suggestion_prefix.clear();
    }
}
