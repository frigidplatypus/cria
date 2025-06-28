use super::state::{App, UndoableAction, PendingAction};
use crate::vikunja::models::Task;

impl App {
    pub fn update_all_tasks(&mut self, tasks: Vec<Task>) {
        self.all_tasks = tasks;
        self.update_label_cache();
        self.apply_task_filter();
        self.selected_task_index = 0;
    }

    pub fn update_label_cache(&mut self) {
        self.label_map.clear();
        self.label_colors.clear();
        for task in &self.all_tasks {
            if let Some(labels) = &task.labels {
                for label in labels {
                    self.label_map.insert(label.id, label.title.clone());
                    self.label_colors.insert(label.id, label.hex_color.clone());
                }
            }
        }
    }

    pub fn toggle_task_completion(&mut self) -> Option<i64> {
        if let Some(task) = self.tasks.get_mut(self.selected_task_index) {
            let task_id = task.id;
            let previous_state = task.done;
            let task_title = task.title.clone();
            task.done = !task.done;
            let new_state = task.done;
            if let Some(all_task) = self.all_tasks.iter_mut().find(|t| t.id == task_id) {
                all_task.done = new_state;
            }
            self.add_to_undo_stack(UndoableAction::TaskCompletion {
                task_id,
                previous_state,
            });
            self.add_debug_message(format!(
                "Task '{}' marked as {}",
                task_title,
                if new_state { "completed" } else { "pending" }
            ));
            if new_state && self.should_hide_completed_immediately() {
                self.apply_task_filter();
                if self.selected_task_index >= self.tasks.len() && !self.tasks.is_empty() {
                    self.selected_task_index = self.tasks.len() - 1;
                }
            }
            self.flash_task_id = Some(task_id);
            self.flash_start = Some(std::time::Instant::now());
            self.flash_cycle_count = 0;
            self.flash_cycle_max = 6;
            Some(task_id)
        } else {
            None
        }
    }

    pub fn request_delete_task(&mut self) {
        if let Some(task) = self.get_selected_task() {
            let task_title = task.title.clone();
            let task_id = task.id;
            self.confirmation_message = format!("Delete task '{}'? This cannot be undone without using undo (U).", task_title);
            self.pending_action = Some(PendingAction::DeleteTask { task_id });
            self.show_confirmation_dialog = true;
        }
    }

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

    pub fn cancel_confirmation(&mut self) {
        self.show_confirmation_dialog = false;
        self.confirmation_message.clear();
        self.pending_action = None;
    }

    fn execute_delete_task(&mut self, task_id: i64) -> Option<i64> {
        if let Some(position) = self.tasks.iter().position(|t| t.id == task_id) {
            let task = self.tasks.remove(position);
            self.add_to_undo_stack(UndoableAction::TaskDeletion {
                task: task.clone(),
                position,
            });
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

    pub fn undo_last_action(&mut self) -> Option<i64> {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                UndoableAction::TaskCompletion { task_id, previous_state } => {
                    if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                        let task_title = task.title.clone();
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
                    let insert_position = position.min(self.tasks.len());
                    self.tasks.insert(insert_position, task.clone());
                    self.selected_task_index = insert_position;
                    self.add_debug_message(format!("Undid deletion of task '{}'", task.title));
                    Some(task.id)
                }
                UndoableAction::TaskCreation { task_id } => {
                    if let Some(position) = self.tasks.iter().position(|t| t.id == task_id) {
                        let task = self.tasks.remove(position);
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

    fn add_to_undo_stack(&mut self, action: UndoableAction) {
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_undo_history {
            self.undo_stack.remove(0);
        }
    }

    pub fn add_task_to_undo_stack(&mut self, task_id: i64) {
        self.add_to_undo_stack(UndoableAction::TaskCreation { task_id });
    }

    pub fn add_task_edit_to_undo_stack(&mut self, task_id: i64, previous_task: Task) {
        self.add_to_undo_stack(UndoableAction::TaskEdit { task_id, previous_task });
    }
}
