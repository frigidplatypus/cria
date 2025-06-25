use crate::vikunja::models::Task;
use std::collections::HashMap;
use chrono::{DateTime, Local, Datelike};

pub struct App {
    pub running: bool,
    pub tasks: Vec<Task>,
    pub project_map: HashMap<i64, String>,
    pub project_colors: HashMap<i64, String>,
    pub selected_task_index: usize,
    pub show_info_pane: bool,
    // Quick Add Modal state
    pub show_quick_add_modal: bool,
    pub quick_add_input: String,
    pub quick_add_cursor_position: usize,
    // Edit Modal state
    pub show_edit_modal: bool,
    pub edit_input: String,
    pub edit_cursor_position: usize,
    pub editing_task_id: Option<i64>,
    // Debug pane state
    pub show_debug_pane: bool,
    pub debug_messages: Vec<(DateTime<Local>, String)>,
    pub show_nerdfont_debug: bool,
}

impl App {
    pub fn new() -> Self {
        Self { 
            running: true, 
            tasks: Vec::new(),
            project_map: HashMap::new(),
            project_colors: HashMap::new(),
            selected_task_index: 0,
            show_info_pane: true,
            show_quick_add_modal: false,
            quick_add_input: String::new(),
            quick_add_cursor_position: 0,
            show_edit_modal: false,
            edit_input: String::new(),
            edit_cursor_position: 0,
            editing_task_id: None,
            show_debug_pane: false,
            debug_messages: Vec::new(),
            show_nerdfont_debug: false,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task_index = (self.selected_task_index + 1) % self.tasks.len();
        }
    }

    pub fn previous_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task_index = if self.selected_task_index == 0 {
                self.tasks.len() - 1
            } else {
                self.selected_task_index - 1
            };
        }
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_index)
    }

    pub fn toggle_info_pane(&mut self) {
        self.show_info_pane = !self.show_info_pane;
    }

    // Quick Add Modal methods
    pub fn show_quick_add_modal(&mut self) {
        self.show_quick_add_modal = true;
        self.quick_add_input.clear();
        self.quick_add_cursor_position = 0;
    }

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

    pub fn get_quick_add_input(&self) -> &str {
        &self.quick_add_input
    }

    pub fn clear_quick_add_input(&mut self) {
        self.quick_add_input.clear();
        self.quick_add_cursor_position = 0;
    }

    // Debug pane methods
    pub fn toggle_debug_pane(&mut self) {
        self.show_debug_pane = !self.show_debug_pane;
    }

    pub fn add_debug_message(&mut self, message: String) {
        self.debug_messages.push((Local::now(), message));
        // Keep only the last 100 messages to prevent memory issues
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
        }
    }

    pub fn clear_debug_messages(&mut self) {
        self.debug_messages.clear();
    }

    // Edit Modal methods
    pub fn show_edit_modal(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let task_id = task.id;
            let magic_syntax = self.task_to_magic_syntax(task);
            
            self.show_edit_modal = true;
            self.editing_task_id = Some(task_id);
            self.edit_input = magic_syntax;
            self.edit_cursor_position = self.edit_input.len();
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

    pub fn get_edit_input(&self) -> &str {
        &self.edit_input
    }

    pub fn clear_edit_input(&mut self) {
        self.edit_input.clear();
        self.edit_cursor_position = 0;
    }

    pub fn toggle_nerdfont_debug(&mut self) {
        self.show_nerdfont_debug = !self.show_nerdfont_debug;
    }

    // Convert task back to Quick Add Magic syntax for editing
    fn task_to_magic_syntax(&self, task: &crate::vikunja::models::Task) -> String {
        let mut result = task.title.clone();
        
        // Add star if favorited
        if task.is_favorite {
            result.push_str(" ^star");
        }
        
        // Add labels
        if let Some(labels) = &task.labels {
            for label in labels {
                result.push_str(&format!(" *{}", label.title));
            }
        }
        
        // Add assignees
        if let Some(assignees) = &task.assignees {
            for assignee in assignees {
                result.push_str(&format!(" @{}", assignee.username));
            }
        }
        
        // Add project (if not the default)
        if let Some(project_name) = self.project_map.get(&task.project_id) {
            if project_name != "Inbox" && task.project_id != 1 {
                result.push_str(&format!(" +{}", project_name));
            }
        }
        
        // Add priority if set
        if let Some(priority) = task.priority {
            if priority > 0 {
                result.push_str(&format!(" !{}", priority));
            }
        }
        
        // Add due date if set
        if let Some(due_date) = &task.due_date {
            // Only show the date if it's a real date (not epoch start)
            if due_date.year() > 1900 {
                // Format the due date in a readable format
                let formatted_date = due_date.format("%Y-%m-%d").to_string();
                result.push_str(&format!(" {}", formatted_date));
            }
        }
        
        result
    }
}
