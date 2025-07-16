use super::state::App;
use crate::tui::app::undoable_action::UndoableAction;
use crate::tui::app::pending_action::PendingAction;
use crate::vikunja::models::Task;

impl App {
    #[allow(dead_code)]
    pub fn toggle_task_completion(&mut self) -> Option<i64> {
        let (task_id, task_title, new_state, previous_state) = if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            let previous_state = task.done;
            let new_state = !task.done;
            let task_id = task.id;
            task.done = new_state;
            (task_id, task.title.clone(), new_state, previous_state)
        } else {
            return None;
        };
        
        // Add to undo stack
        self.add_to_undo_stack(UndoableAction::TaskCompletion { 
            task_id, 
            previous_state 
        });
        
        if new_state {
            self.add_debug_message(format!("Task completed: {}", task_title));
            self.show_toast(format!("Task marked complete: {}", task_title));
        } else {
            self.add_debug_message(format!("Task uncompleted: {}", task_title));
            self.show_toast(format!("Task marked incomplete: {}", task_title));
        }
        Some(task_id)
    }
    pub async fn toggle_star_selected_task_async(&mut self, client: &crate::vikunja_client::VikunjaClient) -> Option<i64> {
        let (task_id, task_title, is_favorite) = if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            task.is_favorite = !task.is_favorite;
            (task.id, task.title.clone(), task.is_favorite)
        } else {
            return None;
        };
        // Call API to update favorite status
        match client.set_task_favorite(task_id as u64, is_favorite).await {
            Ok(_) => {
                self.add_debug_message(format!("Task {}starred: {}", if is_favorite { "" } else { "un" }, task_title));
                self.show_toast(format!("Task {}starred: {}", if is_favorite { "" } else { "un" }, task_title));
            },
            Err(e) => {
                self.add_debug_message(format!("Failed to update favorite: {}", e));
                self.show_toast(format!("Failed to update favorite: {}", e));
            }
        }
        Some(task_id)
    }

    /// Test-only synchronous version of toggle_star_selected_task for unit tests
    #[allow(dead_code)]
    pub fn toggle_star_selected_task(&mut self) -> Option<i64> {
        if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            task.is_favorite = !task.is_favorite;
            let task_id = task.id;
            let task_title = task.title.clone();
            let is_favorite = task.is_favorite;
            self.add_debug_message(format!("Task {}starred: {}", if is_favorite { "" } else { "un" }, task_title));
            self.show_toast(format!("Task {}starred: {}", if is_favorite { "" } else { "un" }, task_title));
            Some(task_id)
        } else {
            None
        }
    }

    pub fn request_delete_task(&mut self) {
        let (show, message, pending) = if let Some(task) = self.get_selected_task() {
            (true, format!("Delete task '{}'?...", task.title), Some(PendingAction::DeleteTask { task_id: task.id }))
        } else {
            (false, String::new(), None)
        };
        self.show_confirmation_dialog = show;
        self.confirmation_message = message;
        self.pending_action = pending;
    }
    pub async fn confirm_action_async(&mut self, client: &crate::vikunja_client::VikunjaClient) -> Option<i64> {
        let action = self.pending_action.take();
        self.show_confirmation_dialog = false;
        if let Some(action) = action {
            match action {
                PendingAction::DeleteTask { task_id } => {
                    self.execute_delete_task_async(task_id, client).await;
                    Some(task_id)
                }
                PendingAction::QuitApp => {
                    self.quit();
                    None
                }
            }
        } else {
            None
        }
    }
    /// Test-only synchronous version of confirm_action for unit tests
    #[allow(dead_code)]
    pub fn confirm_action(&mut self) -> Option<i64> {
        let action = self.pending_action.take();
        self.show_confirmation_dialog = false;
        if let Some(action) = action {
            match action {
                PendingAction::DeleteTask { task_id } => {
                    self.execute_delete_task(task_id);
                    Some(task_id)
                }
                PendingAction::QuitApp => {
                    self.quit();
                    None
                }
            }
        } else {
            None
        }
    }
    pub fn cancel_confirmation(&mut self) { self.show_confirmation_dialog = false; self.pending_action = None; }
    pub async fn execute_delete_task_async(&mut self, task_id: i64, client: &crate::vikunja_client::VikunjaClient) {
        match client.delete_task(task_id).await {
            Ok(_) => {
                if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
                    let task = self.tasks.remove(pos);
                    self.add_debug_message(format!("Task deleted: {}", task.title));
                    self.show_toast(format!("Task deleted: {}", task.title));
                    self.add_to_undo_stack(UndoableAction::TaskDeletion { task, position: pos });
                }
            },
            Err(e) => {
                self.add_debug_message(format!("Failed to delete task {}: {}", task_id, e));
                self.show_toast(format!("Failed to delete task: {}", e));
            }
        }
    }
    /// Test-only synchronous version of execute_delete_task for unit tests
    #[allow(dead_code)]
    pub fn execute_delete_task(&mut self, task_id: i64) { 
        if let Some(pos) = self.tasks.iter().position(|t| t.id == task_id) {
            let task = self.tasks.remove(pos);
            self.add_debug_message(format!("Task deleted: {}", task.title));
            self.show_toast(format!("Task deleted: {}", task.title));
            self.add_to_undo_stack(UndoableAction::TaskDeletion { task, position: pos });
        } 
    }
    #[allow(dead_code)] // Future undo/redo feature
    pub fn undo_last_action(&mut self) -> Option<i64> {
        if let Some(action) = self.undo_stack.pop() {
            let result = match &action {
                UndoableAction::TaskCompletion { task_id, previous_state } => {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                        let task_title = task.title.clone();
                        let current_state = task.done;
                        task.done = *previous_state;
                        self.add_debug_message(format!(
                            "Undid completion toggle for task '{}'", 
                            task_title
                        ));
                        // Push the reverse action to redo stack
                        self.redo_stack.push(UndoableAction::TaskCompletion {
                            task_id: *task_id,
                            previous_state: current_state,
                        });
                        Some(*task_id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskDeletion { task, position } => {
                    let tasks_len = self.tasks.len();
                    let insert_position = (*position).min(tasks_len);
                    self.tasks.insert(insert_position, task.clone());
                    self.selected_task_index = insert_position;
                    self.add_debug_message(format!("Undid deletion of task '{}'", task.title));
                    // Push the reverse action to redo stack
                    self.redo_stack.push(UndoableAction::TaskCreation {
                        task_id: task.id,
                    });
                    Some(task.id)
                }
                UndoableAction::TaskCreation { task_id } => {
                    if let Some(position) = self.tasks.iter().position(|t| t.id == *task_id) {
                        let task = self.tasks.remove(position);
                        if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                            self.selected_task_index = self.tasks.len() - 1;
                        }
                        self.add_debug_message(format!("Undid creation of task '{}'", task.title));
                        // Push the reverse action to redo stack
                        self.redo_stack.push(UndoableAction::TaskDeletion {
                            task: task.clone(),
                            position,
                        });
                        Some(task.id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskEdit { task_id, previous_task } => {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                        let current_task = task.clone();
                        *task = previous_task.clone();
                        self.add_debug_message(format!("Undid edit of task '{}'", previous_task.title));
                        // Push the reverse action to redo stack
                        self.redo_stack.push(UndoableAction::TaskEdit {
                            task_id: *task_id,
                            previous_task: current_task,
                        });
                        Some(*task_id)
                    } else {
                        None
                    }
                }
            };
            
            // Limit redo stack size
            if self.redo_stack.len() > self.max_undo_history {
                self.redo_stack.remove(0);
            }
            
            result
        } else {
            self.add_debug_message("No actions to undo".to_string());
            None
        }
    }
    pub fn redo_last_action(&mut self) -> Option<i64> {
        if let Some(action) = self.redo_stack.pop() {
            let result = match &action {
                UndoableAction::TaskCompletion { task_id, previous_state } => {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                        let task_title = task.title.clone();
                        let current_state = task.done;
                        task.done = *previous_state;
                        self.add_debug_message(format!(
                            "Redid completion toggle for task '{}'", 
                            task_title
                        ));
                        // Push the reverse action to undo stack
                        self.undo_stack.push(UndoableAction::TaskCompletion {
                            task_id: *task_id,
                            previous_state: current_state,
                        });
                        Some(*task_id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskDeletion { task, position } => {
                    let tasks_len = self.tasks.len();
                    let insert_position = (*position).min(tasks_len);
                    self.tasks.insert(insert_position, task.clone());
                    self.selected_task_index = insert_position;
                    self.add_debug_message(format!("Redid deletion of task '{}'", task.title));
                    // Push the reverse action to undo stack
                    self.undo_stack.push(UndoableAction::TaskCreation {
                        task_id: task.id,
                    });
                    Some(task.id)
                }
                UndoableAction::TaskCreation { task_id } => {
                    if let Some(position) = self.tasks.iter().position(|t| t.id == *task_id) {
                        let task = self.tasks.remove(position);
                        if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                            self.selected_task_index = self.tasks.len() - 1;
                        }
                        self.add_debug_message(format!("Redid creation of task '{}'", task.title));
                        // Push the reverse action to undo stack
                        self.undo_stack.push(UndoableAction::TaskDeletion {
                            task: task.clone(),
                            position,
                        });
                        Some(task.id)
                    } else {
                        None
                    }
                }
                UndoableAction::TaskEdit { task_id, previous_task } => {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == *task_id) {
                        let current_task = task.clone();
                        *task = previous_task.clone();
                        self.add_debug_message(format!("Redid edit of task '{}'", previous_task.title));
                        // Push the reverse action to undo stack
                        self.undo_stack.push(UndoableAction::TaskEdit {
                            task_id: *task_id,
                            previous_task: current_task,
                        });
                        Some(*task_id)
                    } else {
                        None
                    }
                }
            };
            
            // Limit undo stack size
            if self.undo_stack.len() > self.max_undo_history {
                self.undo_stack.remove(0);
            }
            
            result
        } else {
            self.add_debug_message("No actions to redo".to_string());
            None
        }
    }
    pub fn add_to_undo_stack(&mut self, action: UndoableAction) { 
        // Clear redo stack when a new action is performed
        self.redo_stack.clear();
        
        if self.undo_stack.len() == self.max_undo_history { 
            self.undo_stack.remove(0); 
        } 
        self.undo_stack.push(action); 
    }
    #[allow(dead_code)] // Future undo/redo feature
    pub fn add_task_to_undo_stack(&mut self, task_id: i64) { if let Some(_task) = self.tasks.iter().find(|t| t.id == task_id) { let action = UndoableAction::TaskCreation { task_id }; self.add_to_undo_stack(action); } }
    #[allow(dead_code)] // Future undo/redo feature
    pub fn add_task_edit_to_undo_stack(&mut self, task_id: i64, previous_task: Task) { let action = UndoableAction::TaskEdit { task_id, previous_task }; self.add_to_undo_stack(action); }
}
