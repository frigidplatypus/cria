use crate::vikunja::models::Task;
use crate::tui::utils::{normalize_string, fuzzy_match_score};
use std::collections::HashMap;
use chrono::{DateTime, Local, Datelike};
use crate::config::CriaConfig;

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

#[derive(Clone, Debug)]
pub enum SuggestionMode {
    Label,
    Project,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // Future sort options
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
    pub config: CriaConfig,
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
    pub redo_stack: Vec<UndoableAction>,
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
    // Quick actions modal state
    pub show_quick_actions_modal: bool, // True when quick actions modal is shown
    pub selected_quick_action_index: usize, // Currently selected quick action in modal
    // Column layout state
    pub current_layout_name: String, // Track current active layout
    // Layout notification state
    pub layout_notification: Option<String>, // Notification message to show
    pub layout_notification_start: Option<std::time::Instant>, // When notification started
}

#[allow(dead_code)]
impl App {
    pub fn new_with_config(config: CriaConfig, default_project_name: String) -> Self {
        let current_layout_name = config.get_active_layout_name();
        Self {
            config,
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
            redo_stack: Vec::new(),
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
            show_quick_actions_modal: false,
            selected_quick_action_index: 0,
            current_layout_name,
            layout_notification: None,
            layout_notification_start: None,
        }
    }

    // --- BEGIN FULL MOVED METHODS ---
    pub fn quit(&mut self) { self.running = false; }
    pub fn next_task(&mut self) { if !self.tasks.is_empty() { self.selected_task_index = (self.selected_task_index + 1) % self.tasks.len(); } }
    pub fn previous_task(&mut self) { if !self.tasks.is_empty() { self.selected_task_index = if self.selected_task_index == 0 { self.tasks.len() - 1 } else { self.selected_task_index - 1 }; } }
    pub fn get_selected_task(&self) -> Option<&Task> { self.tasks.get(self.selected_task_index) }
    pub fn toggle_info_pane(&mut self) { self.show_info_pane = !self.show_info_pane; }
    pub fn show_quick_add_modal(&mut self) { 
        self.close_all_modals();
        self.show_quick_add_modal = true; 
        self.quick_add_input.clear(); 
        self.quick_add_cursor_position = 0; 
    }
    pub fn hide_quick_add_modal(&mut self) { self.show_quick_add_modal = false; self.quick_add_input.clear(); self.quick_add_cursor_position = 0; }
    pub fn add_char_to_quick_add(&mut self, c: char) { self.quick_add_input.insert(self.quick_add_cursor_position, c); self.quick_add_cursor_position += 1; }
    pub fn delete_char_from_quick_add(&mut self) { if self.quick_add_cursor_position > 0 { self.quick_add_cursor_position -= 1; self.quick_add_input.remove(self.quick_add_cursor_position); } }
    pub fn move_cursor_left(&mut self) { if self.quick_add_cursor_position > 0 { self.quick_add_cursor_position -= 1; } }
    pub fn move_cursor_right(&mut self) { if self.quick_add_cursor_position < self.quick_add_input.len() { self.quick_add_cursor_position += 1; } }
    pub fn get_quick_add_input(&self) -> &str { &self.quick_add_input }
    pub fn clear_quick_add_input(&mut self) { self.quick_add_input.clear(); self.quick_add_cursor_position = 0; }
    pub fn toggle_debug_pane(&mut self) { self.show_debug_pane = !self.show_debug_pane; }
    pub fn add_debug_message(&mut self, message: String) {
        use std::fs::OpenOptions;
        use std::io::Write;
        let now = Local::now();
        self.debug_messages.push((now, message.clone()));
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
        }
        let log_line = format!("{}: {}\n", now.format("%Y-%m-%d %H:%M:%S"), message);
        // Only log to file if CRIA_DEBUG is set
        if std::env::var("CRIA_DEBUG").is_ok() {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("cria_debug.log") {
                let _ = file.write_all(log_line.as_bytes());
            }
        }
    }
    pub fn clear_debug_messages(&mut self) { self.debug_messages.clear(); }
    pub fn show_edit_modal(&mut self) { 
        if let Some(task) = self.get_selected_task() { 
            let task_id = task.id; 
            let magic_syntax = self.task_to_magic_syntax(task); 
            self.close_all_modals();
            self.show_edit_modal = true; 
            self.editing_task_id = Some(task_id); 
            self.edit_input = magic_syntax; 
            self.edit_cursor_position = self.edit_input.len(); 
        } 
    }
    pub fn hide_edit_modal(&mut self) { self.show_edit_modal = false; self.edit_input.clear(); self.edit_cursor_position = 0; self.editing_task_id = None; }
    pub fn add_char_to_edit(&mut self, c: char) { self.edit_input.insert(self.edit_cursor_position, c); self.edit_cursor_position += 1; }
    pub fn delete_char_from_edit(&mut self) { if self.edit_cursor_position > 0 { self.edit_cursor_position -= 1; self.edit_input.remove(self.edit_cursor_position); } }
    pub fn move_edit_cursor_left(&mut self) { if self.edit_cursor_position > 0 { self.edit_cursor_position -= 1; } }
    pub fn move_edit_cursor_right(&mut self) { if self.edit_cursor_position < self.edit_input.len() { self.edit_cursor_position += 1; } }
    pub fn get_edit_input(&self) -> &str { &self.edit_input }
    pub fn clear_edit_input(&mut self) { self.edit_input.clear(); self.edit_cursor_position = 0; }
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
    // --- Task manipulation logic moved to tasks.rs ---
    pub fn update_suggestions(&mut self, input: &str, cursor: usize) {
        // Find the last * or + before the cursor
        let before_cursor = &input[..cursor];
        
        // Helper function to check if we're still in a suggestion context
        // We stop suggestions when we encounter certain delimiters or control characters
        fn is_suggestion_char(c: char) -> bool {
            !matches!(c, '\n' | '\r' | '\t' | '#' | '@' | '!' | '&' | '|' | '(' | ')' | '{' | '}' | '"' | '\'')
        }
        
        if let Some(pos) = before_cursor.rfind('*') {
            let after = &before_cursor[pos+1..];
            // Special handling for square brackets - if we're inside [], continue until ]
            let suggestion_text = if after.starts_with('[') {
                &after[1..] // Skip the opening bracket
            } else {
                after
            };
            
            // Allow spaces and more characters in label suggestions, but stop at certain delimiters
            if suggestion_text.chars().all(is_suggestion_char) {
                self.suggestion_mode = Some(SuggestionMode::Label);
                self.suggestion_prefix = suggestion_text.to_string();
                let prefix = suggestion_text.trim();
                let labels: Vec<_> = self.label_map.values().cloned().collect();
                
                // Use fuzzy matching with scoring for better results
                let mut scored_labels: Vec<(String, f32)> = labels.into_iter()
                    .map(|label| {
                        let score = fuzzy_match_score(&label, prefix);
                        (label, score)
                    })
                    .filter(|(_, score)| *score > 0.0)
                    .collect();
                
                // Sort by score (highest first), then alphabetically
                scored_labels.sort_by(|a, b| {
                    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.0.cmp(&b.0))
                });
                
                let filtered: Vec<String> = scored_labels.into_iter()
                    .map(|(label, _)| label)
                    .collect();
                
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
            // Special handling for square brackets - if we're inside [], continue until ]
            let suggestion_text = if after.starts_with('[') {
                &after[1..] // Skip the opening bracket
            } else {
                after
            };
            
            // Allow spaces and more characters in project suggestions, but stop at certain delimiters
            if suggestion_text.chars().all(is_suggestion_char) {
                self.suggestion_mode = Some(SuggestionMode::Project);
                self.suggestion_prefix = suggestion_text.to_string();
                let prefix_lower = suggestion_text.to_lowercase();
                let prefix = prefix_lower.trim();
                let projects: Vec<_> = self.project_map.iter()
                    // Filter out system projects (ID <= 0)
                    .filter(|(id, _)| **id > 0)
                    .map(|(_, name)| name.clone())
                    .collect();
                
                // Use fuzzy matching with scoring for better results
                let mut scored_projects: Vec<(String, f32)> = projects.into_iter()
                    .map(|project| {
                        let score = fuzzy_match_score(&project, prefix);
                        (project, score)
                    })
                    .filter(|(_, score)| *score > 0.0)
                    .collect();
                
                // Sort by score (highest first), then alphabetically
                scored_projects.sort_by(|a, b| {
                    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.0.cmp(&b.0))
                });
                
                let filtered: Vec<String> = scored_projects.into_iter()
                    .map(|(project, _)| project)
                    .collect();
                
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
            SortOrder::TitleAZ => self.tasks.sort_by(|a, b| normalize_string(&a.title).cmp(&normalize_string(&b.title))),
            SortOrder::TitleZA => self.tasks.sort_by(|a, b| normalize_string(&b.title).cmp(&normalize_string(&a.title))),
            SortOrder::PriorityHighToLow => self.tasks.sort_by(|a, b| b.priority.unwrap_or(i32::MIN).cmp(&a.priority.unwrap_or(i32::MIN))),
            SortOrder::PriorityLowToHigh => self.tasks.sort_by(|a, b| a.priority.unwrap_or(i32::MAX).cmp(&b.priority.unwrap_or(i32::MAX))),
            SortOrder::FavoriteStarredFirst => {
                self.tasks.sort_by(|a, b| {
                    let cmp = b.is_favorite.cmp(&a.is_favorite);
                    if cmp == std::cmp::Ordering::Equal {
                        normalize_string(&a.title).cmp(&normalize_string(&b.title))
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
    pub fn show_help_modal(&mut self) {
        self.close_all_modals();
        self.show_help_modal = true;
    }
    
    pub fn hide_help_modal(&mut self) {
        self.show_help_modal = false;
    }
    
    pub fn show_sort_modal(&mut self) {
        self.close_all_modals();
        self.show_sort_modal = true;
    }
    
    pub fn hide_sort_modal(&mut self) {
        self.show_sort_modal = false;
    }

    pub fn show_quick_actions_modal(&mut self) {
        self.close_all_modals();
        self.show_quick_actions_modal = true;
        self.selected_quick_action_index = 0;
    }

    pub fn hide_quick_actions_modal(&mut self) {
        self.show_quick_actions_modal = false;
        self.selected_quick_action_index = 0;
    }

    // Helper method to close all modals
    fn close_all_modals(&mut self) {
        self.show_help_modal = false;
        self.show_sort_modal = false;
        self.show_quick_actions_modal = false;
        self.show_quick_add_modal = false;
        self.show_edit_modal = false;
        // Reset modal state
        self.quick_add_input.clear();
        self.quick_add_cursor_position = 0;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.editing_task_id = None;
        self.selected_quick_action_index = 0;
    }

    // Column layout methods
    pub fn switch_to_next_layout(&mut self) {
        let layouts = self.config.get_column_layouts();
        let old_layout = self.current_layout_name.clone();
        self.current_layout_name = self.config.next_layout(&self.current_layout_name);
        let (layout_name, description) = self.get_current_layout_info();
        let message = if let Some(desc) = description {
            format!("Layout: {} - {} ({})", layout_name, desc, layouts.len())
        } else {
            format!("Layout: {} ({})", layout_name, layouts.len())
        };
        self.show_layout_notification(message);
        // Debug message to see what's happening
        self.add_debug_message(format!("Layout switch: {} -> {} (total: {})", old_layout, layout_name, layouts.len()));
    }

    pub fn switch_to_previous_layout(&mut self) {
        let layouts = self.config.get_column_layouts();
        let old_layout = self.current_layout_name.clone();
        self.current_layout_name = self.config.previous_layout(&self.current_layout_name);
        let (layout_name, description) = self.get_current_layout_info();
        let message = if let Some(desc) = description {
            format!("Layout: {} - {} ({})", layout_name, desc, layouts.len())
        } else {
            format!("Layout: {} ({})", layout_name, layouts.len())
        };
        self.show_layout_notification(message);
        // Debug message to see what's happening
        self.add_debug_message(format!("Layout switch: {} -> {} (total: {})", old_layout, layout_name, layouts.len()));
    }

    pub fn get_current_layout_columns(&self) -> Vec<crate::config::TableColumn> {
        if let Some(layout) = self.config.get_layout(&self.current_layout_name) {
            layout.columns
        } else {
            self.config.get_table_columns()
        }
    }

    pub fn get_current_layout_info(&self) -> (String, Option<String>) {
        if let Some(layout) = self.config.get_layout(&self.current_layout_name) {
            (layout.name, layout.description)
        } else {
            (self.current_layout_name.clone(), None)
        }
    }

    // Quick action methods
    pub fn apply_quick_action(&mut self, action: &crate::config::QuickAction) -> Result<String, String> {
        if self.tasks.is_empty() {
            return Err("No tasks available".to_string());
        }

        let task_index = self.selected_task_index;
        if task_index >= self.tasks.len() {
            return Err("Invalid task selection".to_string());
        }

        match action.action.as_str() {
            "project" => {
                // Find project ID by name
                let project_id = self.project_map
                    .iter()
                    .find_map(|(id, name)| {
                        if name.eq_ignore_ascii_case(&action.target) {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| format!("Project '{}' not found", action.target))?;

                self.tasks[task_index].project_id = project_id;
                Ok(format!("Moved task to project: {}", action.target))
            },
            "priority" => {
                let priority: i32 = action.target.parse()
                    .map_err(|_| format!("Invalid priority: {}", action.target))?;
                
                if priority < 1 || priority > 5 {
                    return Err("Priority must be between 1 and 5".to_string());
                }

                self.tasks[task_index].priority = Some(priority);
                Ok(format!("Set task priority to: {}", priority))
            },
            "label" => {
                // This is more complex as we need to add to existing labels
                // For now, we'll just return a message that this would add the label
                Ok(format!("Would add label: {}", action.target))
            },
            _ => Err(format!("Unknown action: {}", action.action))
        }
    }

    // Layout notification methods
    pub fn show_layout_notification(&mut self, message: String) {
        self.layout_notification = Some(message);
        self.layout_notification_start = Some(std::time::Instant::now());
    }
    
    pub fn get_layout_notification(&self) -> Option<&String> {
        if let (Some(ref notification), Some(start_time)) = (&self.layout_notification, self.layout_notification_start) {
            // Show notification for 3 seconds
            if start_time.elapsed().as_secs() < 3 {
                Some(notification)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn clear_expired_layout_notification(&mut self) {
        if let Some(start_time) = self.layout_notification_start {
            if start_time.elapsed().as_secs() >= 3 {
                self.layout_notification = None;
                self.layout_notification_start = None;
            }
        }
    }
}
