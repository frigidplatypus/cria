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

#[derive(Debug, Clone)]
pub enum SubtaskOperation {
    MakeSubtask, // Make selected task a subtask of another task
    AddSubtask,  // Add a new subtask to the selected task
}

pub struct App {
    pub config: CriaConfig,
    pub running: bool,
    pub tasks: Vec<Task>,
    pub all_tasks: Vec<Task>, // Store all tasks for local filtering
    pub detailed_task_cache: HashMap<i64, Task>, // Cache for detailed task data with comments
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
    pub filter_descriptions: std::collections::HashMap<i64, String>, // Filter descriptions
    pub current_filter_id: Option<i64>,
    // Active project override from filter
    pub active_project_override: Option<String>, // Project name override from current filter
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
    pub show_advanced_help_modal: bool,
    pub show_advanced_features_modal: bool,
    pub selected_advanced_feature_index: usize,
    pub show_sort_modal: bool,
    pub sort_options: Vec<&'static str>,
    pub selected_sort_index: usize,
    pub current_sort: Option<SortOrder>,
    pub show_quick_actions_modal: bool,
    pub selected_quick_action_index: usize,
    // Subtask management
    pub show_subtask_modal: bool,
    pub subtask_operation: Option<SubtaskOperation>,
    pub subtask_picker_input: String,
    pub filtered_subtask_tasks: Vec<(i64, String)>, // (task_id, title)
    pub selected_subtask_picker_index: usize,
    pub selected_subtask_task_ids: Vec<i64>, // For bulk operations
    // Add subtask modal state
    pub show_add_subtask_modal: bool,
    pub add_subtask_input: String,
    pub add_subtask_cursor_position: usize,
    // Quick action mode - direct key handling after Space
    pub quick_action_mode: bool,
    pub quick_action_mode_start: Option<DateTime<Local>>,
    // Attachment modal state
    pub show_attachment_modal: bool,
    pub attachment_modal: Option<crate::tui::modals::AttachmentModal>,
    
    // File picker modal state
    pub show_file_picker_modal: bool,
    pub file_picker_modal: Option<crate::tui::modals::FilePickerModal>,
    
    // URL modal state
    pub show_url_modal: bool,
    pub url_modal: Option<crate::tui::modals::UrlModal>,
    
    // Layout system
    pub current_layout_name: String,
    pub layout_notification: Option<String>,
    pub layout_notification_start: Option<DateTime<Local>>,
    // Toast notifications
    pub toast_notification: Option<String>,
    pub toast_notification_start: Option<DateTime<Local>>,
    pub picker_context: PickerContext,
    // Relation modals - DISABLED: Incomplete feature
    // pub show_relations_modal: bool,
    // pub show_add_relation_modal: bool,
    // pub add_relation_input: String,
    // pub add_relation_cursor_position: usize,
    // pub relations_task_id: Option<i64>,
}

#[allow(dead_code)]
impl App {
    /// Open the label picker from the form editor, preserving form state and context
    pub fn open_label_picker_from_form(&mut self) {
        // Do not close all modals, just hide the form modal
        self.show_form_edit_modal = false;
        self.show_label_picker = true;
        self.label_picker_input.clear();
        self.selected_label_picker_index = 0;
        // Pre-select labels already in the form state
        if let Some(ref form) = self.form_edit_state {
            self.selected_label_ids = form.label_ids.clone();
        }
        self.update_filtered_labels();
        self.picker_context = PickerContext::FormEditLabel;
    }

    /// Open the project picker from the form editor, preserving form state and context
    pub fn open_project_picker_from_form(&mut self) {
        // Do not close all modals, just hide the form modal
        self.show_form_edit_modal = false;
        self.show_project_picker = true;
        self.project_picker_input.clear();
        self.selected_project_picker_index = 0;
        // Pre-select project already in the form state
        if let Some(ref form) = self.form_edit_state {
            self.current_project_id = Some(form.project_id);
        }
        self.update_filtered_projects();
        self.picker_context = PickerContext::FormEditProject;
    }
    // ...existing code...
    // ...existing code...
    pub fn new_with_config(config: CriaConfig, default_project_name: String) -> Self {
        let current_layout_name = config.get_active_layout_name();
        Self {
            config,
            running: true, 
            tasks: Vec::new(),
            all_tasks: Vec::new(),
            detailed_task_cache: HashMap::new(),
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
            filter_descriptions: std::collections::HashMap::new(),
            current_filter_id: None,
            active_project_override: None,
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
            show_advanced_help_modal: false,
            show_advanced_features_modal: false,
            selected_advanced_feature_index: 0,
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
            // Subtask management
            show_subtask_modal: false,
            subtask_operation: None,
            subtask_picker_input: String::new(),
            filtered_subtask_tasks: Vec::new(),
            selected_subtask_picker_index: 0,
            selected_subtask_task_ids: Vec::new(),
            // Add subtask modal
            show_add_subtask_modal: false,
            add_subtask_input: String::new(),
            add_subtask_cursor_position: 0,
            quick_action_mode: false,
            quick_action_mode_start: None,
            show_attachment_modal: false,
            attachment_modal: None,
            show_file_picker_modal: false,
            file_picker_modal: None,
            show_url_modal: false,
            url_modal: None,
            current_layout_name,
            layout_notification: None,
            layout_notification_start: None,
            toast_notification: None,
            toast_notification_start: None,
            picker_context: PickerContext::None,
            // Relation modals - DISABLED: Incomplete feature  
            // show_relations_modal: false,
            // show_add_relation_modal: false,
            // add_relation_input: String::new(),
            // add_relation_cursor_position: 0,
            // relations_task_id: None,
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

    pub fn get_detailed_task(&self, task_id: i64) -> Option<&Task> {
        self.detailed_task_cache.get(&task_id)
    }

    pub fn cache_detailed_task(&mut self, task: Task) {
        self.detailed_task_cache.insert(task.id, task);
    }
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
                // Apply hierarchical sorting for default order to maintain relationships
                self.apply_hierarchical_sort();
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

    pub fn show_attachment_modal(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let attachments = task.attachments.clone().unwrap_or_default();
            let task_title = task.title.clone();
            let task_id = task.id;
            
            self.add_debug_message(format!("Opening attachment modal for task {} (ID: {}) with {} attachments", 
                task_title, task_id, attachments.len()));
            
            self.close_all_modals();
            self.show_attachment_modal = true;
            self.attachment_modal = Some(crate::tui::modals::AttachmentModal::new(
                attachments,
                task_title,
                task_id,
            ));
        } else {
            self.add_debug_message("No task selected for attachment modal".to_string());
        }
    }

    pub fn hide_attachment_modal(&mut self) {
        self.show_attachment_modal = false;
        self.attachment_modal = None;
        self.show_file_picker_modal = false;
        self.file_picker_modal = None;
    }

    pub fn show_file_picker_modal(&mut self) {
        self.show_file_picker_modal = true;
        self.file_picker_modal = Some(crate::tui::modals::FilePickerModal::new(None));
    }

    pub fn hide_file_picker_modal(&mut self) {
        self.show_file_picker_modal = false;
        self.file_picker_modal = None;
    }

    pub fn show_url_modal(&mut self, urls: Vec<crate::url_utils::UrlWithContext>) {
        if urls.is_empty() {
            return;
        }
        self.close_all_modals();
        self.show_url_modal = true;
        self.url_modal = Some(crate::tui::modals::UrlModal::new(urls));
    }

    pub fn hide_url_modal(&mut self) {
        self.show_url_modal = false;
        self.url_modal = None;
    }

    pub fn show_advanced_help_modal(&mut self) {
        self.close_all_modals();
        self.show_advanced_help_modal = true;
    }

    pub fn hide_advanced_help_modal(&mut self) {
        self.show_advanced_help_modal = false;
    }

    pub fn show_advanced_features_modal(&mut self) {
        self.close_all_modals();
        self.show_advanced_features_modal = true;
        self.selected_advanced_feature_index = 0;
    }

    pub fn hide_advanced_features_modal(&mut self) {
        self.show_advanced_features_modal = false;
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
        self.show_advanced_help_modal = false;
        self.show_advanced_features_modal = false;
        self.show_sort_modal = false;
        self.show_quick_actions_modal = false;
        self.show_quick_add_modal = false;
        self.show_edit_modal = false;
        self.show_form_edit_modal = false;
        self.show_project_picker = false;
        self.show_filter_picker = false;
        self.show_confirmation_dialog = false;
        self.show_attachment_modal = false;
        self.show_file_picker_modal = false;
        self.show_url_modal = false;
        self.show_add_subtask_modal = false;
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
        self.attachment_modal = None;
        self.file_picker_modal = None;
        self.url_modal = None;
        self.add_subtask_input.clear();
        self.add_subtask_cursor_position = 0;
        // Relations modals - DISABLED: Incomplete feature
        // self.show_relations_modal = false;
        // self.show_add_relation_modal = false;
        // self.add_relation_input.clear();
        // self.add_relation_cursor_position = 0;
        // self.relations_task_id = None;
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
        // Note: The actual refresh logic is handled in ui_loop.rs when the 'r' key is pressed
        // This method just sets the flag to indicate a refresh is needed
    }

    /// Applies the edit modal's input to the selected task (simple title update for demonstration)
    pub fn apply_edit_modal(&mut self) {
        if let Some(idx) = self.tasks.get(self.selected_task_index).map(|_| self.selected_task_index) {
            // For demonstration, just update the title to the edit_input
            // In a real app, you'd parse the magic syntax and update all fields
            self.tasks[idx].title = self.edit_input.clone();
        }
    }
    
    pub fn set_filters_with_descriptions(&mut self, filters: Vec<(i64, String)>, descriptions: std::collections::HashMap<i64, String>) {
        self.filters = filters;
        self.filter_descriptions = descriptions;
        self.update_filtered_filters();
    }

    // Subtask management methods
    pub fn show_subtask_modal(&mut self, operation: SubtaskOperation) {
        self.show_subtask_modal = true;
        self.subtask_operation = Some(operation);
        self.subtask_picker_input.clear();
        self.selected_subtask_picker_index = 0;
        self.update_filtered_subtask_tasks();
    }

    pub fn hide_subtask_modal(&mut self) {
        self.show_subtask_modal = false;
        self.subtask_operation = None;
        self.subtask_picker_input.clear();
        self.filtered_subtask_tasks.clear();
        self.selected_subtask_picker_index = 0;
        self.selected_subtask_task_ids.clear();
    }

    // Add subtask modal methods
    pub fn show_add_subtask_modal(&mut self) {
        self.close_all_modals();
        self.show_add_subtask_modal = true;
        self.add_subtask_input.clear();
        self.add_subtask_cursor_position = 0;
    }

    pub fn hide_add_subtask_modal(&mut self) {
        self.show_add_subtask_modal = false;
        self.add_subtask_input.clear();
        self.add_subtask_cursor_position = 0;
    }

    pub fn add_char_to_add_subtask(&mut self, c: char) {
        self.add_subtask_input.insert(self.add_subtask_cursor_position, c);
        self.add_subtask_cursor_position += 1;
    }

    pub fn delete_char_from_add_subtask(&mut self) {
        if self.add_subtask_cursor_position > 0 {
            self.add_subtask_cursor_position -= 1;
            self.add_subtask_input.remove(self.add_subtask_cursor_position);
        }
    }

    pub fn move_add_subtask_cursor_left(&mut self) {
        if self.add_subtask_cursor_position > 0 {
            self.add_subtask_cursor_position -= 1;
        }
    }

    pub fn move_add_subtask_cursor_right(&mut self) {
        if self.add_subtask_cursor_position < self.add_subtask_input.len() {
            self.add_subtask_cursor_position += 1;
        }
    }

    pub fn get_add_subtask_input(&self) -> &str {
        &self.add_subtask_input
    }

    pub fn add_char_to_subtask_input(&mut self, c: char) {
        self.subtask_picker_input.push(c);
        self.update_filtered_subtask_tasks();
    }

    pub fn delete_char_from_subtask_input(&mut self) {
        self.subtask_picker_input.pop();
        self.update_filtered_subtask_tasks();
    }

    pub fn next_subtask_task(&mut self) {
        if !self.filtered_subtask_tasks.is_empty() && self.selected_subtask_picker_index < self.filtered_subtask_tasks.len() - 1 {
            self.selected_subtask_picker_index += 1;
        }
    }

    pub fn previous_subtask_task(&mut self) {
        if self.selected_subtask_picker_index > 0 {
            self.selected_subtask_picker_index -= 1;
        }
    }

    fn update_filtered_subtask_tasks(&mut self) {
        if self.subtask_picker_input.is_empty() {
            // Show all tasks except the currently selected one
            self.filtered_subtask_tasks = self.all_tasks
                .iter()
                .filter(|task| {
                    if let Some(selected_task) = self.get_selected_task() {
                        task.id != selected_task.id
                    } else {
                        true
                    }
                })
                .map(|task| (task.id, task.title.clone()))
                .collect();
        } else {
            let input_normalized = normalize_string(&self.subtask_picker_input);
            let mut scored_tasks: Vec<_> = self.all_tasks
                .iter()
                .filter(|task| {
                    if let Some(selected_task) = self.get_selected_task() {
                        task.id != selected_task.id
                    } else {
                        true
                    }
                })
                .filter_map(|task| {
                    let title_normalized = normalize_string(&task.title);
                    let score = fuzzy_match_score(&input_normalized, &title_normalized);
                    if score > 0.0 {
                        Some((task.id, task.title.clone(), score))
                    } else {
                        None
                    }
                })
                .collect();
            
            scored_tasks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            self.filtered_subtask_tasks = scored_tasks.into_iter().map(|(id, title, _)| (id, title)).collect();
        }
        
        // Reset selection to first item
        self.selected_subtask_picker_index = 0;
    }

    pub fn get_selected_subtask_task(&self) -> Option<(i64, String)> {
        self.filtered_subtask_tasks.get(self.selected_subtask_picker_index).cloned()
    }

    /// Toggle selection of current subtask task (for bulk operations)
    pub fn toggle_subtask_task_selection(&mut self) {
        if let Some((task_id, _)) = self.get_selected_subtask_task() {
            if self.selected_subtask_task_ids.contains(&task_id) {
                self.selected_subtask_task_ids.retain(|&id| id != task_id);
            } else {
                self.selected_subtask_task_ids.push(task_id);
            }
        }
    }

    /// Check if a subtask task is selected for bulk operations
    pub fn is_subtask_task_selected(&self, task_id: i64) -> bool {
        self.selected_subtask_task_ids.contains(&task_id)
    }

    /// Get visual indicator for subtask relationships
    pub fn get_task_relation_indicator(&self, task: &crate::vikunja::models::Task) -> Option<&'static str> {
        if let Some(ref related_tasks) = task.related_tasks {
            // Check if this task is a subtask of another
            if related_tasks.contains_key("subtask") && !related_tasks["subtask"].is_empty() {
                // Check if any parent task is still active (not done)
                let has_active_parent = related_tasks["subtask"]
                    .iter()
                    .any(|parent_task| !parent_task.done);
                
                if has_active_parent {
                    return Some("  └─"); // Subtask indicator with indentation
                }
                // If all parent tasks are done, don't show subtask indicator
            }
            // Check if this task has subtasks
            if related_tasks.contains_key("parenttask") && !related_tasks["parenttask"].is_empty() {
                return Some("├─"); // Parent task indicator
            }
        }
        None
    }

    /// Apply hierarchical sorting to maintain parent-child relationships
    pub fn apply_hierarchical_sort(&mut self) {
        // First, identify all parent-child relationships
        let mut parent_child_map: HashMap<i64, Vec<i64>> = HashMap::new();
        let mut child_parent_map: HashMap<i64, i64> = HashMap::new();
        
        // Build the relationship maps
        for task in &self.tasks {
            if let Some(ref related_tasks) = task.related_tasks {
                // If this task has subtasks (is a parent)
                if let Some(subtasks) = related_tasks.get("subtask") {
                    let subtask_ids: Vec<i64> = subtasks.iter().map(|t| t.id).collect();
                    parent_child_map.insert(task.id, subtask_ids.clone());
                    
                    // Also populate the reverse mapping
                    for subtask_id in subtask_ids {
                        child_parent_map.insert(subtask_id, task.id);
                    }
                }
            }
        }
        
        // Create a new sorted list
        let mut sorted_tasks = Vec::new();
        let mut processed_tasks = std::collections::HashSet::new();
        
        // First pass: Add all parent tasks and their children in order
        for task in &self.tasks {
            // Skip if already processed
            if processed_tasks.contains(&task.id) {
                continue;
            }
            
            // Check if this is a parent task (has subtasks)
            if parent_child_map.contains_key(&task.id) {
                // Add the parent task
                sorted_tasks.push(task.clone());
                processed_tasks.insert(task.id);
                
                // Add all its subtasks immediately after
                if let Some(subtask_ids) = parent_child_map.get(&task.id) {
                    for subtask_id in subtask_ids {
                        if let Some(subtask) = self.tasks.iter().find(|t| t.id == *subtask_id) {
                            sorted_tasks.push(subtask.clone());
                            processed_tasks.insert(*subtask_id);
                        }
                    }
                }
            }
            // Check if this is an orphaned subtask (parent not in current view)
            else if child_parent_map.contains_key(&task.id) {
                // Only add if parent is not in the current task list
                let parent_id = child_parent_map[&task.id];
                if !self.tasks.iter().any(|t| t.id == parent_id) {
                    sorted_tasks.push(task.clone());
                    processed_tasks.insert(task.id);
                }
            }
            // This is a standalone task (neither parent nor child)
            else {
                sorted_tasks.push(task.clone());
                processed_tasks.insert(task.id);
            }
        }
        
        self.tasks = sorted_tasks;
    }

    /// Get the hierarchical display info for a task (indentation level and prefix)
    pub fn get_task_hierarchy_info(&self, task: &crate::vikunja::models::Task) -> (usize, &'static str) {
        if let Some(ref related_tasks) = task.related_tasks {
            // Check if this task is a subtask of another (has parenttask relations)
            if related_tasks.contains_key("parenttask") && !related_tasks["parenttask"].is_empty() {
                // Check if any parent task is still active (not done)
                let has_active_parent = related_tasks["parenttask"]
                    .iter()
                    .any(|parent_task| !parent_task.done);
                
                if has_active_parent {
                    return (1, "└─ "); // Level 1 indentation with subtask indicator
                }
                // If all parent tasks are done, don't show subtask indicator
            }
            // Check if this task has subtasks (has subtask relations)
            if related_tasks.contains_key("subtask") && !related_tasks["subtask"].is_empty() {
                return (0, ""); // No indentation, no prefix for parent tasks
            }
        }
        (0, "") // No hierarchy indicator
    }

    /// Check if a task is a subtask of another task
    pub fn is_subtask_of(&self, task: &crate::vikunja::models::Task, parent_id: i64) -> bool {
        if let Some(ref related_tasks) = task.related_tasks {
            if let Some(parent_tasks) = related_tasks.get("subtask") {
                return parent_tasks.iter().any(|parent| parent.id == parent_id);
            }
        }
        false
    }

    /// Get all subtasks of a given task
    pub fn get_subtasks_of(&self, parent_id: i64) -> Vec<&crate::vikunja::models::Task> {
        self.tasks.iter()
            .filter(|task| self.is_subtask_of(task, parent_id))
            .collect()
    }

    /// Get the parent task of a subtask (if any)
    pub fn get_parent_of(&self, task: &crate::vikunja::models::Task) -> Option<&crate::vikunja::models::Task> {
        if let Some(ref related_tasks) = task.related_tasks {
            if let Some(parent_tasks) = related_tasks.get("subtask") {
                if let Some(parent_task) = parent_tasks.first() {
                    return self.tasks.iter().find(|t| t.id == parent_task.id);
                }
            }
        }
        None
    }

    // Relations methods - DISABLED: Incomplete feature
    // pub fn hide_relations_modal(&mut self) { self.show_relations_modal = false; }
    // pub fn show_add_relation_modal(&mut self) { self.show_add_relation_modal = true; }
    // pub fn hide_add_relation_modal(&mut self) { self.show_add_relation_modal = false; }
    // pub fn next_relation_kind(&mut self) { /* stub */ }
    // pub fn previous_relation_kind(&mut self) { /* stub */ }
    // pub fn add_char_to_relation_input(&mut self, c: char) { self.add_relation_input.push(c); }
    // pub fn delete_char_from_relation_input(&mut self) { self.add_relation_input.pop(); }
    // pub fn move_relation_cursor_left(&mut self) { if self.add_relation_cursor_position > 0 { self.add_relation_cursor_position -= 1; } }
    // pub fn move_relation_cursor_right(&mut self) { self.add_relation_cursor_position += 1; }
    // pub fn get_selected_relation_kind(&self) -> Option<RelationKind> { None }
    // pub fn get_task_relation_indicator(&self, _task: &crate::vikunja::models::Task) -> Option<RelationKind> {
    //     // Return a valid variant for testing, e.g. Precedes
    //     Some(RelationKind::Precedes)
    // }
}
