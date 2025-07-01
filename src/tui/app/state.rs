use crate::vikunja::models::Task;
use std::collections::HashMap;
use chrono::{DateTime, Local, Datelike};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum TaskFilter {
    ActiveOnly,    // Hide completed tasks (default)
    All,          // Show all tasks
    CompletedOnly, // Show only completed tasks
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum UndoableAction {
    TaskCompletion { 
        task_id: i64, 
        previous_state: bool 
    },
    TaskDeletion { 
        task: Task, 
        position: usize 
    },
    TaskCreation { 
        task_id: i64 
    },
    TaskEdit { 
        task_id: i64, 
        previous_task: Task 
    },
}

#[derive(Clone, Debug)]
pub enum PendingAction {
    DeleteTask { task_id: i64 },
}

pub enum SuggestionMode {
    Label,
    Project,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Default,
    TitleAZ,
    TitleZA,
    PriorityHighToLow,
    PriorityLowToHigh,
    FavoriteStarredFirst, // NEW: Sort by favorite status (starred first)
    DueDateEarliestFirst, // Sort by due date (earliest first)
    DueDateLatestFirst,   // Sort by due date (latest first)
    StartDateEarliestFirst, // Sort by start date (earliest first)
    StartDateLatestFirst,   // Sort by start date (latest first)
}

pub struct App {
    pub running: bool,
    pub tasks: Vec<Task>,
    pub all_tasks: Vec<Task>, // Store all tasks for local filtering
    pub project_map: HashMap<i64, String>,
    pub project_colors: HashMap<i64, String>,
    pub label_map: HashMap<i64, String>,
    pub label_colors: HashMap<i64, String>,
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
    // Undo system
    pub undo_stack: Vec<UndoableAction>,
    pub max_undo_history: usize,
    // Confirmation dialog state
    pub show_confirmation_dialog: bool,
    pub confirmation_message: String,
    pub pending_action: Option<PendingAction>,
    // Task filtering
    pub task_filter: TaskFilter,
    // Project picker modal state
    pub show_project_picker: bool,
    pub project_picker_input: String,
    pub filtered_projects: Vec<(i64, String)>, // (project_id, name)
    pub selected_project_picker_index: usize,
    pub current_project_id: Option<i64>,
    // Saved filter picker modal state
    pub show_filter_picker: bool,
    pub filter_picker_input: String,
    pub filters: Vec<(i64, String)>, // (filter_id, title)
    pub filtered_filters: Vec<(i64, String)>,
    pub selected_filter_picker_index: usize,
    pub current_filter_id: Option<i64>,
    pub refreshing: bool, // Indicates if a refresh is in progress
    // Flash effect for task row
    pub flash_task_id: Option<i64>,
    pub flash_start: Option<std::time::Instant>,
    pub flash_cycle_count: u8, // Number of completed flash cycles
    pub flash_cycle_max: u8,   // Max cycles to flash
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub suggestion_mode: Option<SuggestionMode>,
    pub suggestion_prefix: String,
    pub default_project_name: String, // NEW: store config default project name
    pub show_help_modal: bool, // Help modal state
    // Sorting modal state
    pub show_sort_modal: bool,
    pub sort_options: Vec<&'static str>,
    pub selected_sort_index: usize,
    pub current_sort: Option<SortOrder>,
}

#[allow(dead_code)]
impl App {
    pub fn new_with_default_project(default_project_name: String) -> Self {
        let app = Self {
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
            flash_cycle_max: 6, // 3 full on/off cycles (6 states)
            suggestions: Vec::new(),
            selected_suggestion: 0,
            suggestion_mode: None,
            suggestion_prefix: String::new(),
            default_project_name,
            show_help_modal: false,
            show_sort_modal: false,
            sort_options: vec![
                "Default (API order)",
                "Title A-Z",
                "Title Z-A",
                "Priority (high to low)",
                "Priority (low to high)",
                "Favorite (starred first)",
                "Due date (earliest first)",
                "Due date (latest first)",
                "Start date (earliest first)",
                "Start date (latest first)",
            ],
            selected_sort_index: 0,
            current_sort: None,
        };
        app
    }

    // --- BEGIN FULL MOVED METHODS ---
    pub fn quit(&mut self) { self.running = false; }
    pub fn next_task(&mut self) { if !self.tasks.is_empty() { self.selected_task_index = (self.selected_task_index + 1) % self.tasks.len(); } }
    pub fn previous_task(&mut self) { if !self.tasks.is_empty() { self.selected_task_index = if self.selected_task_index == 0 { self.tasks.len() - 1 } else { self.selected_task_index - 1 }; } }
    pub fn get_selected_task(&self) -> Option<&Task> { self.tasks.get(self.selected_task_index) }
    pub fn toggle_info_pane(&mut self) { self.show_info_pane = !self.show_info_pane; }
    pub fn show_quick_add_modal(&mut self) { self.show_quick_add_modal = true; self.quick_add_input.clear(); self.quick_add_cursor_position = 0; }
    pub fn hide_quick_add_modal(&mut self) { self.show_quick_add_modal = false; self.quick_add_input.clear(); self.quick_add_cursor_position = 0; }
    pub fn add_char_to_quick_add(&mut self, c: char) { self.quick_add_input.insert(self.quick_add_cursor_position, c); self.quick_add_cursor_position += 1; }
    pub fn delete_char_from_quick_add(&mut self) { if self.quick_add_cursor_position > 0 { self.quick_add_cursor_position -= 1; self.quick_add_input.remove(self.quick_add_cursor_position); } }
    pub fn move_cursor_left(&mut self) { if self.quick_add_cursor_position > 0 { self.quick_add_cursor_position -= 1; } }
    pub fn move_cursor_right(&mut self) { if self.quick_add_cursor_position < self.quick_add_input.len() { self.quick_add_cursor_position += 1; } }
    pub fn get_quick_add_input(&self) -> &str { &self.quick_add_input }
    pub fn clear_quick_add_input(&mut self) { self.quick_add_input.clear(); self.quick_add_cursor_position = 0; }
    pub fn toggle_debug_pane(&mut self) { self.show_debug_pane = !self.show_debug_pane; }
    pub fn add_debug_message(&mut self, message: String) { use std::fs::OpenOptions; use std::io::Write; let now = Local::now(); self.debug_messages.push((now, message.clone())); if self.debug_messages.len() > 100 { self.debug_messages.remove(0); } let log_line = format!("{}: {}\n", now.format("%Y-%m-%d %H:%M:%S"), message); if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("cria_debug.log") { let _ = file.write_all(log_line.as_bytes()); } }
    pub fn clear_debug_messages(&mut self) { self.debug_messages.clear(); }
    pub fn show_edit_modal(&mut self) { if let Some(task) = self.get_selected_task() { let task_id = task.id; let magic_syntax = self.task_to_magic_syntax(task); self.show_edit_modal = true; self.editing_task_id = Some(task_id); self.edit_input = magic_syntax; self.edit_cursor_position = self.edit_input.len(); } }
    pub fn hide_edit_modal(&mut self) { self.show_edit_modal = false; self.edit_input.clear(); self.edit_cursor_position = 0; self.editing_task_id = None; }
    pub fn add_char_to_edit(&mut self, c: char) { self.edit_input.insert(self.edit_cursor_position, c); self.edit_cursor_position += 1; }
    pub fn delete_char_from_edit(&mut self) { if self.edit_cursor_position > 0 { self.edit_cursor_position -= 1; self.edit_input.remove(self.edit_cursor_position); } }
    pub fn move_edit_cursor_left(&mut self) { if self.edit_cursor_position > 0 { self.edit_cursor_position -= 1; } }
    pub fn move_edit_cursor_right(&mut self) { if self.edit_cursor_position < self.edit_input.len() { self.edit_cursor_position += 1; } }
    pub fn get_edit_input(&self) -> &str { &self.edit_input }
    pub fn clear_edit_input(&mut self) { self.edit_input.clear(); self.edit_cursor_position = 0; }
    fn task_to_magic_syntax(&self, task: &crate::vikunja::models::Task) -> String { let mut result = task.title.clone(); if task.is_favorite { result.push_str(" ^star"); } if let Some(labels) = &task.labels { for label in labels { result.push_str(&format!(" *{}", label.title)); } } if let Some(assignees) = &task.assignees { for assignee in assignees { result.push_str(&format!(" @{}", assignee.username)); } } if let Some(project_name) = self.project_map.get(&task.project_id) { if project_name != "Inbox" && task.project_id != 1 { result.push_str(&format!(" +{}", project_name)); } } if let Some(priority) = task.priority { if priority > 0 { result.push_str(&format!(" !{}", priority)); } } if let Some(due_date) = &task.due_date { if due_date.year() > 1900 { let formatted_date = due_date.format("%Y-%m-%d").to_string(); result.push_str(&format!(" {}", formatted_date)); } } result }
    // --- Task manipulation logic moved to tasks.rs ---
    pub fn update_suggestions(&mut self, input: &str, cursor: usize) {
        // Find the last * or + before the cursor
        let before_cursor = &input[..cursor];
        if let Some(pos) = before_cursor.rfind('*') {
            let after = &before_cursor[pos+1..];
            if after.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                self.suggestion_mode = Some(SuggestionMode::Label);
                self.suggestion_prefix = after.to_string();
                let prefix = after.to_lowercase();
                let mut labels: Vec<_> = self.label_map.values().cloned().collect();
                labels.sort();
                let filtered: Vec<_> = labels.into_iter().filter(|l| l.to_lowercase().starts_with(&prefix)).collect();
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
                self.suggestion_mode = Some(SuggestionMode::Project);
                self.suggestion_prefix = after.to_string();
                let prefix = after.to_lowercase();
                let mut projects: Vec<_> = self.project_map.iter()
                    .filter(|(id, _)| **id > 0)
                    .map(|(_, name)| name.clone())
                    .collect();
                projects.sort();
                let filtered: Vec<_> = projects.into_iter().filter(|p| p.to_lowercase().starts_with(&prefix)).collect();
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
    pub fn jump_to_top(&mut self) {
        self.selected_task_index = 0;
    }
    pub fn jump_to_bottom(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task_index = self.tasks.len() - 1;
        }
    }
    pub fn apply_sort(&mut self, sort: SortOrder) {
        self.current_sort = Some(sort.clone());
        match sort {
            SortOrder::Default => {
                let ids: Vec<i64> = self.tasks.iter().map(|t| t.id).collect();
                let mut new_tasks = Vec::new();
                for t in &self.all_tasks {
                    if ids.contains(&t.id) {
                        new_tasks.push(t.clone());
                    }
                }
                self.tasks = new_tasks;
            }
            SortOrder::TitleAZ => self.tasks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase())),
            SortOrder::TitleZA => self.tasks.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase())),
            SortOrder::PriorityHighToLow => self.tasks.sort_by(|a, b| b.priority.unwrap_or(i32::MIN).cmp(&a.priority.unwrap_or(i32::MIN))),
            SortOrder::PriorityLowToHigh => self.tasks.sort_by(|a, b| a.priority.unwrap_or(i32::MAX).cmp(&b.priority.unwrap_or(i32::MAX))),
            SortOrder::FavoriteStarredFirst => {
                self.tasks.sort_by(|a, b| {
                    let cmp = b.is_favorite.cmp(&a.is_favorite);
                    if cmp == std::cmp::Ordering::Equal {
                        a.title.to_lowercase().cmp(&b.title.to_lowercase())
                    } else {
                        cmp
                    }
                });
            }
            SortOrder::DueDateEarliestFirst => {
                self.tasks.sort_by(|a, b| a.due_date.cmp(&b.due_date));
            }
            SortOrder::DueDateLatestFirst => {
                self.tasks.sort_by(|a, b| b.due_date.cmp(&a.due_date));
            }
            SortOrder::StartDateEarliestFirst => {
                self.tasks.sort_by(|a, b| a.start_date.cmp(&b.start_date));
            }
            SortOrder::StartDateLatestFirst => {
                self.tasks.sort_by(|a, b| b.start_date.cmp(&a.start_date));
            }
        }
    }
    pub fn hide_help_modal(&mut self) {
        self.show_help_modal = false;
    }
    pub fn hide_sort_modal(&mut self) {
        self.show_sort_modal = false;
    }
}
