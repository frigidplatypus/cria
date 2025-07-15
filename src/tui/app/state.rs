use crate::vikunja::models::Task;
use crate::tui::utils::{normalize_string, fuzzy_match_score};
use std::collections::HashMap;
use chrono::{DateTime, Local, Datelike};
use crate::config::CriaConfig;
use crate::tui::app::form_edit_state::FormEditState;
use crate::tui::app::sort_order::SortOrder;
use crate::tui::app::picker_context::PickerContext;
use crate::tui::app::task_filter::TaskFilter;
use crate::tui::app::undoable_action::UndoableAction;
use crate::tui::app::pending_action::PendingAction;
use crate::tui::app::suggestion_mode::SuggestionMode;

mod confirm_quit_ext;

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
    // Form Edit Modal state
    pub show_form_edit_modal: bool,
    pub form_edit_state: Option<FormEditState>,
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
    // Label picker modal state
    pub show_label_picker: bool,
    pub label_picker_input: String,
    pub filtered_labels: Vec<(i64, String)>, // (label_id, title)
    pub selected_label_picker_index: usize,
    pub selected_label_ids: Vec<i64>, // Currently selected labels
    // Filter picker modal state
    pub show_filter_picker: bool,
    pub filter_picker_input: String,
    pub filtered_filters: Vec<(i64, String)>, // (filter_id, title)
    pub selected_filter_picker_index: usize,
    pub filters: Vec<(i64, String)>, // Available filters
    pub current_filter_id: Option<i64>,
    // Flash feedback state
    pub refreshing: bool,
    pub flash_task_id: Option<i64>,
    pub flash_start: Option<DateTime<Local>>,
    pub flash_cycle_count: usize,
    pub flash_cycle_max: usize,
    // Suggestion system
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
    pub suggestion_mode: Option<SuggestionMode>,
    pub suggestion_prefix: String,
    // Default project
    pub default_project_name: String,
    // Modal states
    pub show_help_modal: bool,
    pub show_sort_modal: bool,
    pub sort_options: Vec<&'static str>,
    pub selected_sort_index: usize,
    pub current_sort: Option<SortOrder>,
    pub show_quick_actions_modal: bool,
    pub selected_quick_action_index: usize,
    // Quick action mode - direct key handling after Space
    pub quick_action_mode: bool,
    pub quick_action_mode_start: Option<DateTime<Local>>,
    // Layout system
    pub current_layout_name: String,
    pub layout_notification: Option<String>,
    pub layout_notification_start: Option<DateTime<Local>>,
    // Toast notifications
    pub toast_notification: Option<String>,
    pub toast_notification_start: Option<DateTime<Local>>,
    pub picker_context: PickerContext,
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
            show_form_edit_modal: false,
            form_edit_state: None,
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
            show_label_picker: false,
            label_picker_input: String::new(),
            filtered_labels: Vec::new(),
            selected_label_picker_index: 0,
            selected_label_ids: Vec::new(),
            show_filter_picker: false,
            filter_picker_input: String::new(),
            filtered_filters: Vec::new(),
            selected_filter_picker_index: 0,
            filters: Vec::new(),
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
            quick_action_mode: false,
            quick_action_mode_start: None,
            current_layout_name,
            layout_notification: None,
            layout_notification_start: None,
            toast_notification: None,
            toast_notification_start: None,
            picker_context: PickerContext::None,
        }
    }

    pub fn default() -> Self {
        Self::new_with_config(CriaConfig::default(), "Inbox".to_string())
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
    
    pub fn show_form_edit_modal(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let form_state = FormEditState::new(task);
            self.close_all_modals();
            self.show_form_edit_modal = true;
            self.form_edit_state = Some(form_state);
        }
    }
    
    pub fn hide_form_edit_modal(&mut self) {
        self.show_form_edit_modal = false;
        self.form_edit_state = None;
    }
    
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

    pub fn enter_quick_action_mode(&mut self) {
        self.close_all_modals();
        self.quick_action_mode = true;
        self.quick_action_mode_start = Some(chrono::Local::now());
    }

    pub fn exit_quick_action_mode(&mut self) {
        self.quick_action_mode = false;
        self.quick_action_mode_start = None;
    }

    pub fn is_quick_action_mode_expired(&self) -> bool {
        if let Some(start_time) = self.quick_action_mode_start {
            chrono::Local::now().signed_duration_since(start_time).num_seconds() >= 2
        } else {
            false
        }
    }

    // Helper method to close all modals
    pub fn close_all_modals(&mut self) {
        self.show_help_modal = false;
        self.show_sort_modal = false;
        self.show_quick_actions_modal = false;
        self.show_quick_add_modal = false;
        self.show_edit_modal = false;
        self.show_form_edit_modal = false;
        self.show_project_picker = false;
        self.show_filter_picker = false;
        self.show_confirmation_dialog = false;
        self.quick_action_mode = false;
        self.quick_action_mode_start = None;
        // Reset modal state
        self.quick_add_input.clear();
        self.quick_add_cursor_position = 0;
        self.edit_input.clear();
        self.edit_cursor_position = 0;
        self.editing_task_id = None;
        self.form_edit_state = None;
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
        
        // Apply layout-specific sort if defined
        self.apply_layout_sort();
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
        
        // Apply layout-specific sort if defined
        self.apply_layout_sort();
    }

    /// Extract and apply layout-specific sort configuration
    pub fn apply_layout_sort(&mut self) {
        if let Some(layout) = self.config.get_layout(&self.current_layout_name) {
            let mut sort_columns: Vec<(&crate::config::TableColumn, &crate::config::ColumnSort)> = layout
                .columns
                .iter()
                .filter_map(|col| col.sort.as_ref().map(|sort| (col, sort)))
                .collect();
            
            // Sort by order (primary sort = 1, secondary = 2, etc.)
            sort_columns.sort_by_key(|(_, sort)| sort.order);
            
            if !sort_columns.is_empty() {
                self.add_debug_message(format!("Applying layout sort with {} levels", sort_columns.len()));
                self.apply_multi_level_sort(&sort_columns);
                // Clear manual sort when layout sort is applied
                self.current_sort = None;
            }
        }
    }

    /// Apply multi-level sorting based on column configuration
    fn apply_multi_level_sort(&mut self, sort_columns: &[(&crate::config::TableColumn, &crate::config::ColumnSort)]) {
        use crate::config::{TaskColumn, SortDirection};
        
        self.tasks.sort_by(|a, b| {
            for (column, sort_config) in sort_columns {
                let ordering = match column.column_type {
                    TaskColumn::Title => {
                        let cmp = normalize_string(&a.title).cmp(&normalize_string(&b.title));
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        }
                    }
                    TaskColumn::Project => {
                        let a_project = self.project_map.get(&a.project_id)
                            .map(|p| p.as_str())
                            .unwrap_or("");
                        let b_project = self.project_map.get(&b.project_id)
                            .map(|p| p.as_str())
                            .unwrap_or("");
                        let cmp = a_project.cmp(b_project);
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        }
                    }
                    TaskColumn::Priority => {
                        // None (no priority) should always sort last, regardless of direction
                        let cmp = match (a.priority, b.priority) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (None, Some(_)) => std::cmp::Ordering::Greater, // None always last
                            (Some(_), None) => std::cmp::Ordering::Less,    // None always last
                            (Some(a_prio), Some(b_prio)) => match sort_config.direction {
                                SortDirection::Asc => a_prio.cmp(&b_prio),
                                SortDirection::Desc => b_prio.cmp(&a_prio),
                            },
                        };
                        cmp
                    }
                    TaskColumn::DueDate => {
                        // None (no due date) should always sort last
                        let cmp = match (&a.due_date, &b.due_date) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (None, Some(_)) => std::cmp::Ordering::Greater, // None always last
                            (Some(_), None) => std::cmp::Ordering::Less,    // None always last
                            (Some(a_due), Some(b_due)) => a_due.cmp(b_due),
                        };
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => match cmp {
                                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater, // Keep None last
                                std::cmp::Ordering::Less => std::cmp::Ordering::Less,       // Keep None last
                                std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                            },
                        }
                    }
                    TaskColumn::StartDate => {
                        // None (no start date) should always sort last
                        let cmp = match (&a.start_date, &b.start_date) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (None, Some(_)) => std::cmp::Ordering::Greater, // None always last
                            (Some(_), None) => std::cmp::Ordering::Less,    // None always last
                            (Some(a_start), Some(b_start)) => a_start.cmp(b_start),
                        };
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => match cmp {
                                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater, // Keep None last
                                std::cmp::Ordering::Less => std::cmp::Ordering::Less,       // Keep None last
                                std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                            },
                        }
                    }
                    TaskColumn::Created => {
                        // Task.created is Option<String>, need to handle comparison
                        let cmp = match (&a.created, &b.created) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (None, Some(_)) => std::cmp::Ordering::Greater, // None always last
                            (Some(_), None) => std::cmp::Ordering::Less,    // None always last
                            (Some(a_created), Some(b_created)) => a_created.cmp(b_created),
                        };
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => match cmp {
                                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater, // Keep None last
                                std::cmp::Ordering::Less => std::cmp::Ordering::Less,       // Keep None last
                                std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                            },
                        }
                    }
                    TaskColumn::Updated => {
                        // Task.updated is Option<String>, need to handle comparison
                        let cmp = match (&a.updated, &b.updated) {
                            (None, None) => std::cmp::Ordering::Equal,
                            (None, Some(_)) => std::cmp::Ordering::Greater, // None always last
                            (Some(_), None) => std::cmp::Ordering::Less,    // None always last
                            (Some(a_updated), Some(b_updated)) => a_updated.cmp(b_updated),
                        };
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => match cmp {
                                std::cmp::Ordering::Greater => std::cmp::Ordering::Greater, // Keep None last
                                std::cmp::Ordering::Less => std::cmp::Ordering::Less,       // Keep None last
                                std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
                            },
                        }
                    }
                    TaskColumn::Status => {
                        let cmp = a.done.cmp(&b.done);
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        }
                    }
                    // For columns that don't have meaningful sort (Labels, Assignees), 
                    // fall back to title sort
                    _ => {
                        let cmp = normalize_string(&a.title).cmp(&normalize_string(&b.title));
                        match sort_config.direction {
                            SortDirection::Asc => cmp,
                            SortDirection::Desc => cmp.reverse(),
                        }
                    }
                };
                
                // If this level produces a non-equal result, use it
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
                // Otherwise, continue to the next sort level
            }
            
            // If all sort levels are equal, maintain stable sort
            std::cmp::Ordering::Equal
        });
    }

    /// Get current layout name and description
    pub fn get_current_layout_info(&self) -> (String, Option<String>) {
        if let Some(layout) = self.config.get_layout(&self.current_layout_name) {
            (layout.name.clone(), layout.description.clone())
        } else {
            (self.current_layout_name.clone(), None)
        }
    }

    /// Show layout notification message
    pub fn show_layout_notification(&mut self, message: String) {
        self.layout_notification = Some(message);
        self.layout_notification_start = Some(Local::now());
    }

    /// Get layout notification if active and within display duration
    pub fn get_layout_notification(&self) -> Option<&String> {
        if let (Some(ref notification), Some(start_time)) = (&self.layout_notification, self.layout_notification_start) {
            // Show notification for 2 seconds
            if Local::now().signed_duration_since(start_time).num_seconds() < 2 {
                Some(notification)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Show toast notification message
    pub fn show_toast(&mut self, message: String) {
        self.toast_notification = Some(message);
        self.toast_notification_start = Some(Local::now());
    }

    /// Get toast notification if active and within display duration
    pub fn get_toast(&self) -> Option<&String> {
        if let (Some(ref notification), Some(start_time)) = (&self.toast_notification, self.toast_notification_start) {
            // Show toast for 2 seconds
            if Local::now().signed_duration_since(start_time).num_seconds() < 2 {
                Some(notification)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Clear toast notification (for use in tick handler)
    pub fn clear_toast(&mut self) {
        self.toast_notification = None;
        self.toast_notification_start = None;
    }

    /// Get current layout columns for rendering
    pub fn get_current_layout_columns(&self) -> Vec<crate::config::TableColumn> {
        if let Some(layout) = self.config.get_layout(&self.current_layout_name) {
            layout.columns.clone()
        } else {
            // Fallback to default columns if layout not found
            self.config.get_columns()
        }
    }

    pub fn apply_quick_action(&mut self, action: &crate::config::QuickAction) -> Result<(), String> {
        if self.tasks.is_empty() {
            return Err("No tasks available".to_string());
        }
        let task = self.tasks.get_mut(self.selected_task_index).ok_or("No selected task")?;
        let task_id = task.id; // Get the task ID before we modify it
        
        let result = match action.action.as_str() {
            "project" => {
                // Find project by name
                let project_id = self.project_map.iter().find_map(|(id, name)| {
                    if name == &action.target { Some(*id) } else { None }
                });
                if let Some(pid) = project_id {
                    task.project_id = pid;
                    Ok(())
                } else {
                    Err(format!("Project '{}' not found", action.target))
                }
            },
            "priority" => {
                if let Ok(priority) = action.target.parse::<i32>() {
                    if (1..=5).contains(&priority) {
                        task.priority = Some(priority);
                        Ok(())
                    } else {
                        Err(format!("Invalid priority '{}': must be 1-5", action.target))
                    }
                } else {
                    Err(format!("Invalid priority '{}': not a number", action.target))
                }
            },
            "label" => {
                // Find label by name
                let label_id = self.label_map.iter().find_map(|(id, name)| {
                    if name == &action.target { Some(*id) } else { None }
                });
                if let Some(lid) = label_id {
                    if let Some(ref mut labels) = task.labels {
                        if !labels.iter().any(|l| l.id == lid) {
                            labels.push(crate::vikunja::models::Label {
                                id: lid,
                                title: action.target.clone(),
                                hex_color: self.label_colors.get(&lid).cloned(),
                                description: None,
                                created: None,
                                updated: None,
                                created_by: None,
                            });
                        }
                    } else {
                        task.labels = Some(vec![crate::vikunja::models::Label {
                            id: lid,
                            title: action.target.clone(),
                            hex_color: self.label_colors.get(&lid).cloned(),
                            description: None,
                            created: None,
                            updated: None,
                            created_by: None,
                        }]);
                    }
                    Ok(())
                } else {
                    Err(format!("Label '{}' not found", action.target))
                }
            },
            _ => Err(format!("Unknown quick action: {}", action.action)),
        };
        
        // If the quick action was successful, also update the corresponding task in all_tasks
        if result.is_ok() {
            if let Some(all_task) = self.all_tasks.iter_mut().find(|t| t.id == task_id) {
                // Copy the updated fields from the filtered task to all_tasks
                match action.action.as_str() {
                    "project" => all_task.project_id = task.project_id,
                    "priority" => all_task.priority = task.priority,
                    "label" => all_task.labels = task.labels.clone(),
                    _ => {}
                }
            }
        }
        
        result
    }

    pub fn cycle_filter_backward(&mut self) {
        if self.filters.is_empty() { return; }
        let idx = match self.current_filter_id {
            Some(id) => self.filters.iter().position(|(fid, _)| *fid == id).unwrap_or(0),
            None => 0,
        };
        let new_idx = if idx == 0 { self.filters.len() - 1 } else { idx - 1 };
        self.current_filter_id = Some(self.filters[new_idx].0);
        self.selected_filter_picker_index = new_idx;
    }
    pub fn cycle_filter_forward(&mut self) {
        if self.filters.is_empty() { return; }
        let idx = match self.current_filter_id {
            Some(id) => self.filters.iter().position(|(fid, _)| *fid == id).unwrap_or(0),
            None => 0,
        };
        let new_idx = if idx + 1 >= self.filters.len() { 0 } else { idx + 1 };
        self.current_filter_id = Some(self.filters[new_idx].0);
        self.selected_filter_picker_index = new_idx;
    }
    pub fn refresh_all(&mut self) {
        self.refreshing = true;
        // This should trigger a reload of tasks, projects, filters, etc. in the main event loop
        self.add_debug_message("Refreshing all data (tasks, projects, filters)".to_string());
    }

    /// Applies the edit modal's input to the selected task (simple title update for demonstration)
    pub fn apply_edit_modal(&mut self) {
        if let Some(idx) = self.tasks.get(self.selected_task_index).map(|_| self.selected_task_index) {
            // For demonstration, just update the title to the edit_input
            // In a real app, you'd parse the magic syntax and update all fields
            self.tasks[idx].title = self.edit_input.clone();
        }
    }
}
