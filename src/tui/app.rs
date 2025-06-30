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
        use std::fs::OpenOptions;
        use std::io::Write;
        let now = Local::now();
        self.debug_messages.push((now, message.clone()));
        // Keep only the last 100 messages to prevent memory issues
        if self.debug_messages.len() > 100 {
            self.debug_messages.remove(0);
        }
        // Write to logfile
        let log_line = format!("{}: {}\n", now.format("%Y-%m-%d %H:%M:%S"), message);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("cria_debug.log") {
            let _ = file.write_all(log_line.as_bytes());
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
    
    // Task completion toggle
    pub fn toggle_task_completion(&mut self) -> Option<i64> {
        if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            let task_id = task.id;
            let previous_state = task.done;
            let task_title = task.title.clone(); // Clone title before other borrows
            
            // Toggle the completion state
            task.done = !task.done;
            let new_state = task.done;
            
            // Also update in all_tasks
            if let Some(all_task) = self.all_tasks.iter_mut().find(|t| t.id == task_id) {
                all_task.done = new_state;
            }
            
            // Add to undo stack
            self.add_to_undo_stack(UndoableAction::TaskCompletion {
                task_id,
                previous_state,
            });
            
            self.add_debug_message(format!(
                "Task '{}' marked as {}", 
                task_title, 
                if new_state { "completed" } else { "pending" }
            ));
            
            // If we just completed a task and we're hiding completed tasks, reapply filter
            if new_state && self.should_hide_completed_immediately() {
                self.apply_task_filter();
                // Adjust selection if the completed task was hidden
                if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                    self.selected_task_index = self.tasks.len() - 1;
                }
            }

            // Flash the toggled row
            self.flash_task_id = Some(task_id);
            self.flash_start = Some(std::time::Instant::now());
            self.flash_cycle_count = 0;
            self.flash_cycle_max = 6;
            
            Some(task_id)
        } else {
            None
        }
    }
    
    // Task deletion with confirmation
    pub fn request_delete_task(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let task_title = task.title.clone();
            let task_id = task.id;
            
            self.confirmation_message = format!("Delete task '{}'? This cannot be undone without using undo (U).", task_title);
            self.pending_action = Some(PendingAction::DeleteTask { task_id });
            self.show_confirmation_dialog = true;
        }
    }
    
    // Confirm and execute pending action
    pub fn confirm_action(&mut self) -> Option<i64> {
        if let Some(action) = self.pending_action.take() {
            match action {
                PendingAction::DeleteTask { task_id } => {
                    self.execute_delete_task(task_id)
                }
            }
        } else {
            None
        }
    }
    
    // Cancel confirmation dialog
    pub fn cancel_confirmation(&mut self) {
        self.show_confirmation_dialog = false;
        self.confirmation_message.clear();
        self.pending_action = None;
    }
    
    // Execute task deletion
    fn execute_delete_task(&mut self, task_id: i64) -> Option<i64> {
        if let Some(position) = self.tasks.iter().position(|t| t.id == task_id) {
            let task = self.tasks.remove(position);
            
            // Add to undo stack
            self.add_to_undo_stack(UndoableAction::TaskDeletion {
                task: task.clone(),
                position,
            });
            
            // Adjust selected index if necessary
            if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                self.selected_task_index = self.tasks.len() - 1;
            }
            
            self.add_debug_message(format!("Task '{}' deleted", task.title));
            self.cancel_confirmation();
            
            Some(task_id)
        } else {
            self.cancel_confirmation();
            None
        }
    }
    
    // Undo last action
    pub fn undo_last_action(&mut self) -> Option<i64> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoableAction::TaskCompletion { task_id, previous_state } => {
                    // Find and restore the task's completion state
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        let task_title = task.title.clone(); // Clone title before modifying
                        task.done = previous_state;
                        self.add_debug_message(format!(
                            "Undid completion toggle for task '{}'", 
                            task_title
                        ));
                        Some(task_id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskDeletion { task, position } => {
                    // Restore the deleted task at its original position
                    let insert_position = position.min(self.tasks.len());
                    self.tasks.insert(insert_position, task.clone());
                    
                    // Update selected index to the restored task
                    self.selected_task_index = insert_position;
                    
                    self.add_debug_message(format!("Undid deletion of task '{}'", task.title));
                    Some(task.id)
                }
                UndoableAction::TaskCreation { task_id } => {
                    // Remove the created task
                    if let Some(position) = self.tasks.iter().position(|t| t.id == task_id) {
                        let task = self.tasks.remove(position);
                        
                        // Adjust selected index if necessary
                        if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                            self.selected_task_index = self.tasks.len() - 1;
                        }
                        
                        self.add_debug_message(format!("Undid creation of task '{}'", task.title));
                        Some(task_id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskEdit { task_id, previous_task } => {
                    // Restore the previous version of the task
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        *task = previous_task.clone();
                        self.add_debug_message(format!("Undid edit of task '{}'", previous_task.title));
                        Some(task_id)
                    } else {
                        None
                    }
                }
            }
        } else {
            self.add_debug_message("No actions to undo".to_string());
            None
        }
    }
    
    // Add action to undo stack
    fn add_to_undo_stack(&mut self, action: UndoableAction) {
        self.undo_stack.push(action);
        
        // Limit undo history
        if self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.remove(0);
        }
    }
    
    // Add to undo stack when creating tasks
    pub fn add_task_to_undo_stack(&mut self, task_id: i64) {
        self.add_to_undo_stack(UndoableAction::TaskCreation { task_id });
    }
    
    // Add to undo stack when editing tasks  
    pub fn add_task_edit_to_undo_stack(&mut self, task_id: i64, previous_task: Task) {
        self.add_to_undo_stack(UndoableAction::TaskEdit { task_id, previous_task });
    }

    // Get confirmation dialog message
    pub fn get_confirmation_message(&self) -> &str {
        &self.confirmation_message
    }
    
    // Task filtering methods
    pub fn cycle_task_filter(&mut self) {
        self.task_filter = match self.task_filter {
            TaskFilter::ActiveOnly => TaskFilter::All,
            TaskFilter::All => TaskFilter::CompletedOnly,
            TaskFilter::CompletedOnly => TaskFilter::ActiveOnly,
        };
        
        self.apply_task_filter();
        self.selected_task_index = 0; // Reset selection
        
        let filter_name = match self.task_filter {
            TaskFilter::ActiveOnly => "Active Tasks Only",
            TaskFilter::All => "All Tasks",
            TaskFilter::CompletedOnly => "Completed Tasks Only",
        };
        
        self.add_debug_message(format!("Switched to filter: {}", filter_name));
    }
    
    pub fn apply_task_filter(&mut self) {
        self.tasks = match self.task_filter {
            TaskFilter::ActiveOnly => {
                self.all_tasks.iter().filter(|task| !task.done).cloned().collect()
            },
            TaskFilter::All => {
                self.all_tasks.clone()
            },
            TaskFilter::CompletedOnly => {
                self.all_tasks.iter().filter(|task| task.done).cloned().collect()
            },
        };
        
        // Adjust selected index if it's out of bounds
        if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_task_index = self.tasks.len() - 1;
        }
    }
    
    /// Update label cache from all tasks
    pub fn update_label_cache(&mut self) {
        self.label_map.clear();
        self.label_colors.clear();
        for task in &self.all_tasks {
            if let Some(labels) = &task.labels {
                for label in labels {
                    self.label_map.insert(label.id, label.title.clone());
                    self.label_colors.insert(label.id, label.hex_color.clone().unwrap_or_default());
                }
            }
        }
    }

    pub fn update_all_tasks(&mut self, tasks: Vec<Task>) {
        self.all_tasks = tasks;
        self.update_label_cache();
        self.apply_task_filter();
        self.selected_task_index = 0;
    }
    
    pub fn get_filter_display_name(&self) -> &str {
        match self.task_filter {
            TaskFilter::ActiveOnly => "Active Only",
            TaskFilter::All => "All Tasks", 
            TaskFilter::CompletedOnly => "Completed Only",
        }
    }
    
    pub fn should_hide_completed_immediately(&self) -> bool {
        matches!(self.task_filter, TaskFilter::ActiveOnly)
    }

    // Project picker modal methods
    pub fn show_project_picker(&mut self) {
        self.show_project_picker = true;
        self.project_picker_input.clear();
        self.update_filtered_projects();
        self.selected_project_picker_index = 0;
    }

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
        let query = self.project_picker_input.to_lowercase();
        let mut projects: Vec<_> = self.project_map.iter()
            .filter(|(id, _)| **id > 0)
            .map(|(id, name)| (*id, name.clone()))
            .collect();
        if !query.is_empty() {
            projects.retain(|(_, name)| name.to_lowercase().contains(&query));
        }
        projects.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        if query.is_empty() {
            self.filtered_projects = vec![(-1, "All Projects".to_string())];
            self.filtered_projects.extend(projects);
        } else {
            self.filtered_projects = projects;
        }
    }

    pub fn apply_project_filter(&mut self) {
        if let Some(pid) = self.current_project_id {
            // Filter by project AND current task filter (e.g., hide completed)
            self.tasks = self.all_tasks.iter()
                .filter(|t| t.project_id == pid)
                .filter(|t| match self.task_filter {
                    TaskFilter::ActiveOnly => !t.done,
                    TaskFilter::All => true,
                    TaskFilter::CompletedOnly => t.done,
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

    // Saved filter picker modal methods
    pub fn show_filter_picker(&mut self) {
        self.show_filter_picker = true;
        self.filter_picker_input.clear();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }

    pub fn hide_filter_picker(&mut self) {
        self.show_filter_picker = false;
        self.filter_picker_input.clear();
    }

    pub fn add_char_to_filter_picker(&mut self, c: char) {
        self.filter_picker_input.push(c);
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }

    pub fn delete_char_from_filter_picker(&mut self) {
        self.filter_picker_input.pop();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }

    pub fn move_filter_picker_up(&mut self) {
        if self.selected_filter_picker_index > 0 {
            self.selected_filter_picker_index -= 1;
        }
    }

    pub fn move_filter_picker_down(&mut self) {
        if self.selected_filter_picker_index + 1 < self.filtered_filters.len() {
            self.selected_filter_picker_index += 1;
        }
    }

    pub fn select_filter_picker(&mut self) {
        if let Some((id, _)) = self.filtered_filters.get(self.selected_filter_picker_index) {
            if *id == -1 {
                self.current_filter_id = None;
            } else {
                self.current_filter_id = Some(*id);
            }
            self.apply_filter();
            self.hide_filter_picker();
        }
    }

    pub fn update_filtered_filters(&mut self) {
        let query = self.filter_picker_input.to_lowercase();
        // Show all filters (negative IDs) and allow search
        let mut filters: Vec<_> = self.filters.iter()
            .map(|(id, title)| (*id, title.clone()))
            .collect();
        if !query.is_empty() {
            filters.retain(|(_, title)| title.to_lowercase().contains(&query));
        }
        filters.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        self.filtered_filters = filters;
    }

    pub fn apply_filter(&mut self) {
        if let Some(fid) = self.current_filter_id {
            // Apply the saved filter logic here
            // This is a placeholder for the actual filter application code
            self.add_debug_message(format!("Applied filter ID: {}", fid));
        } else {
            self.apply_task_filter();
        }
        self.selected_task_index = 0;
    }

    pub fn get_current_filter_name(&self) -> String {
        if let Some(fid) = self.current_filter_id {
            self.filters.iter().find(|(id, _)| *id == fid).map(|(_, title)| title.clone()).unwrap_or_else(|| "Unknown Filter".to_string())
        } else {
            "No Filter".to_string()
        }
    }

    /// Set the list of saved filters (called after fetching from API or project_map)
    pub fn set_filters(&mut self, filters: Vec<(i64, String)>) {
        // Only keep negative IDs (filter views)
        self.filters = filters.into_iter().filter(|(id, _)| *id < 0).collect();
        self.update_filtered_filters();
        self.selected_filter_picker_index = 0;
    }

    // Update tasks after fetching for a saved filter
    pub fn apply_filter_tasks(&mut self, tasks: Vec<Task>) {
        self.all_tasks = tasks;
        self.apply_task_filter();
        self.selected_task_index = 0;
    }

    pub fn toggle_star_selected_task(&mut self) -> Option<i64> {
        if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            task.is_favorite = !task.is_favorite;
            return Some(task.id);
        }
        None
    }

    /// Update suggestions based on current input and cursor position
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
                // Only reset selected_suggestion if suggestions changed
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

    /// Jump to the first task
    pub fn jump_to_top(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task_index = 0;
        }
    }
    /// Jump to the last task
    pub fn jump_to_bottom(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task_index = self.tasks.len() - 1;
        }
    }

    // Help modal methods
    pub fn show_help_modal(&mut self) {
        self.show_help_modal = true;
    }
    pub fn hide_help_modal(&mut self) {
        self.show_help_modal = false;
    }

    // Sorting modal methods
    pub fn show_sort_modal(&mut self) {
        self.show_sort_modal = true;
    }
    pub fn hide_sort_modal(&mut self) {
        self.show_sort_modal = false;
    }
    pub fn apply_sort(&mut self, sort: SortOrder) {
        self.current_sort = Some(sort.clone());
        match sort {
            SortOrder::Default => {
                // Restore tasks to the order in all_tasks, filtered as currently displayed
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
                // Sort by favorite status (starred first), then by title
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
}
