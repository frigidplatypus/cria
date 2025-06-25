use crate::vikunja::models::Task;
use std::collections::HashMap;
use chrono::{DateTime, Local};

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
    // Debug pane state
    pub show_debug_pane: bool,
    pub debug_messages: Vec<(DateTime<Local>, String)>,
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
            show_debug_pane: false,
            debug_messages: Vec::new(),
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
}
