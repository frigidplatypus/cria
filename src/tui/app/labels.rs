use crate::tui::app::{App, PickerContext};
use crate::tui::utils::contains_ignore_case;

impl App {
    pub fn show_label_picker(&mut self) {
        self.close_all_modals();
        self.show_label_picker = true;
        self.label_picker_input.clear();
        self.selected_label_picker_index = 0;
        self.update_filtered_labels();
    }
    
    pub fn hide_label_picker(&mut self) {
        self.show_label_picker = false;
        self.label_picker_input.clear();
    }
    
    pub fn add_char_to_label_picker(&mut self, c: char) {
        self.label_picker_input.push(c);
        self.update_filtered_labels();
        self.selected_label_picker_index = 0; // Reset selection to first item
    }
    
    pub fn delete_char_from_label_picker(&mut self) {
        if !self.label_picker_input.is_empty() {
            self.label_picker_input.pop();
            self.update_filtered_labels();
            self.selected_label_picker_index = 0; // Reset selection to first item
        }
    }
    
    pub fn move_label_picker_up(&mut self) {
        if !self.filtered_labels.is_empty() {
            self.selected_label_picker_index = (self.selected_label_picker_index + self.filtered_labels.len() - 1) % self.filtered_labels.len();
        }
    }
    
    pub fn move_label_picker_down(&mut self) {
        if !self.filtered_labels.is_empty() {
            self.selected_label_picker_index = (self.selected_label_picker_index + 1) % self.filtered_labels.len();
        }
    }
    
    pub fn toggle_label_picker(&mut self) {
        if let Some(label) = self.filtered_labels.get(self.selected_label_picker_index) {
            let label_id = label.0;
            if self.selected_label_ids.contains(&label_id) {
                // Remove if already selected
                self.selected_label_ids.retain(|&id| id != label_id);
            } else {
                // Add if not selected
                self.selected_label_ids.push(label_id);
            }
            
            // Update the form edit state if it exists
            if let Some(ref mut form_state) = self.form_edit_state {
                form_state.label_ids = self.selected_label_ids.clone();
            }
        }
    }
    
    pub fn select_label_picker(&mut self) {
        let picker_context = self.picker_context.clone();
        self.hide_label_picker();
        if let PickerContext::FormEditLabel = picker_context {
            if let Some(ref mut form) = self.form_edit_state {
                // Set label_ids to selected_label_ids
                form.label_ids = self.selected_label_ids.clone();
            }
            self.show_form_edit_modal = true;
            self.picker_context = crate::tui::app::PickerContext::None;
        }
    }
    
    pub fn update_filtered_labels(&mut self) {
        let query = &self.label_picker_input;
        self.filtered_labels = self.label_map.iter()
            .filter(|(_, name)| contains_ignore_case(name, query))
            .map(|(id, name)| (*id, name.clone()))
            .collect::<Vec<_>>();
    }
}
