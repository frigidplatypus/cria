use crate::vikunja::models::Task;
use std::collections::HashMap;
use chrono::{DateTime, Local, Datelike};

#[derive(Clone, Debug)]
pub enum TaskFilter {
    ActiveOnly,
    All,
    CompletedOnly,
}

#[derive(Clone, Debug)]
pub enum UndoableAction {
    TaskCompletion { task_id: i64, previous_state: bool },
    TaskDeletion { task: Task, position: usize },
    TaskCreation { task_id: i64 },
    TaskEdit { task_id: i64, previous_task: Task },
}

#[derive(Clone, Debug)]
pub enum PendingAction {
    DeleteTask { task_id: i64 },
}

pub enum SuggestionMode {
    Label,
    Project,
}

pub struct App {
    pub running: bool,
    pub tasks: Vec<Task>,
    pub all_tasks: Vec<Task>,
    pub project_map: HashMap<i64, String>,
    pub project_colors: HashMap<i64, String>,
    pub label_map: HashMap<i64, String>,
    pub label_colors: HashMap<i64, String>,
    pub selected_task_index: usize,
    pub show_info_pane: bool,
    pub show_quick_add_modal: bool,
    pub quick_add_input: String,
    pub quick_add_cursor_position: usize,
    pub show_edit_modal: bool,
    pub edit_input: String,
    pub edit_cursor_position: usize,
    pub editing_task_id: Option<i64>,
    pub show_debug_pane: bool,
    pub debug_messages: Vec<(DateTime<Local>, String)>,
    pub show_nerdfont_debug: bool,
    pub undo_stack: Vec<UndoableAction>,
    pub max_undo_history: usize,
    pub show_confirmation_dialog: bool,
    pub confirmation_message: String,
    pub pending_action: Option<PendingAction>,
    pub task_filter: TaskFilter,
    pub show_project_picker: bool,
    pub project_picker_input: String,
    pub filtered_projects: Vec<(i64, String)>,
    pub selected_project_picker_index: usize,
    pub current_project_id: Option<i64>,
    pub show_filter_picker: bool,
    pub filter_picker_input: String,
    pub filters: Vec<(i64, String)>,
    pub filtered_filters: Vec<(i64, String)>,
    pub selected_filter_picker_index: usize,
    pub current_filter_id: Option<i64>,
    pub refreshing: bool,
    pub flash_task_id: Option<i64>,
    pub flash_start: Option<std::time::Instant>,
    pub flash_cycle_count: u8,
    pub flash_cycle_max: u8,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub suggestion_mode: Option<SuggestionMode>,
    pub suggestion_prefix: String,
    pub show_keybinds_modal: bool,
    pub project_picker_assign_to_task: bool, // true if assigning project to a task
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            tasks: Vec::new(),
            all_tasks: Vec::new(),
            project_map: HashMap::new(),
            project_colors: HashMap::new(),
            label_map: HashMap::new(),
            label_colors: HashMap::new(),
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
            undo_stack: Vec::new(),
            max_undo_history: 50,
            show_confirmation_dialog: false,
            confirmation_message: String::new(),
            pending_action: None,
            task_filter: TaskFilter::ActiveOnly,
            show_project_picker: false,
            project_picker_input: String::new(),
            filtered_projects: Vec::new(),
            selected_project_picker_index: 0,
            current_project_id: None,
            show_filter_picker: false,
            filter_picker_input: String::new(),
            filters: Vec::new(),
            filtered_filters: Vec::new(),
            selected_filter_picker_index: 0,
            current_filter_id: None,
            refreshing: false,
            flash_task_id: None,
            flash_start: None,
            flash_cycle_count: 0,
            flash_cycle_max: 6,
            suggestions: Vec::new(),
            selected_suggestion: 0,
            suggestion_mode: None,
            suggestion_prefix: String::new(),
            show_keybinds_modal: false,
            project_picker_assign_to_task: false, // Initialize to false
        }
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_task_index)
    }
    pub fn get_quick_add_input(&self) -> &str {
        &self.quick_add_input
    }
    pub fn get_edit_input(&self) -> &str {
        &self.edit_input
    }
    pub fn add_debug_message(&mut self, message: String) {
        crate::tui::utils::debug_log(self, &message);
        let now = chrono::Local::now();
        self.debug_messages.push((now, message));
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
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
    pub fn set_filters(&mut self, filters: Vec<(i64, String)>) {
        self.filters = filters.into_iter().filter(|(id, _)| *id < 0).collect();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }
    pub fn show_filter_picker(&mut self) {
        self.show_filter_picker = true;
        self.filter_picker_input.clear();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }
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
    pub fn show_project_picker(&mut self) {
        self.show_project_picker = true;
        self.project_picker_input.clear();
        self.update_filtered_projects();
        self.selected_project_picker_index = 0;
    }
    pub fn toggle_star_selected_task(&mut self) -> Option<i64> {
        if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            task.is_favorite = !task.is_favorite;
            return Some(task.id);
        }
        None
    }
    fn task_to_magic_syntax(&self, task: &crate::vikunja::models::Task) -> String {
        let mut result = task.title.clone();
        if task.is_favorite {
            result.push_str(" ^star");
        }
        if let Some(labels) = &task.labels {
            for label in labels {
                result.push_str(&format!(" *{}", label.title));
            }
        }
        if let Some(assignees) = &task.assignees {
            for assignee in assignees {
                result.push_str(&format!(" @{}", assignee.username));
            }
        }
        if let Some(project_name) = self.project_map.get(&task.project_id) {
            if project_name != "Inbox" && task.project_id != 1 {
                result.push_str(&format!(" +{}", project_name));
            }
        }
        if let Some(priority) = task.priority {
            if priority > 0 {
                result.push_str(&format!(" !{}", priority));
            }
        }
        if let Some(due_date) = &task.due_date {
            if due_date.year() > 1900 {
                let formatted_date = due_date.format("%Y-%m-%d").to_string();
                result.push_str(&format!(" {}", formatted_date));
            }
        }
        result
    }
}
