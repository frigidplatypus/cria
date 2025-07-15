use crate::tui::app::state::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;
use chrono::Local;
use crate::tui::app::pending_action::PendingAction;

// Confirmation dialog handler
pub async fn handle_confirmation_dialog(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            // Confirm the action
            if let Some(pending_action) = app.pending_action.take() {
                match pending_action {
                    PendingAction::DeleteTask { task_id } => {
                        debug_log(&format!("Confirmed delete for task ID: {}", task_id));
                        // Call delete API and refresh tasks (existing logic)
                        let api_client_guard = api_client.lock().await;
                        if let Err(e) = api_client_guard.delete_task(task_id).await {
                            debug_log(&format!("ERROR: Failed to delete task from API: {}", e));
                        } else {
                            debug_log(&format!("Task {} deleted from API", task_id));
                        }
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        app.apply_task_filter();
                        if let Some(task) = app.tasks.get(app.selected_task_index) {
                            app.flash_task_id = Some(task.id);
                            app.flash_start = Some(Local::now());
                        }
                        app.flash_cycle_count = 0;
                        app.flash_cycle_max = 6;
                        app.show_confirmation_dialog = false;
                    }
                    PendingAction::QuitApp => {
                        app.quit();
                        app.show_confirmation_dialog = false;
                    }
                }
            }
        },
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            // Cancel the action
            app.cancel_confirmation();
            app.show_confirmation_dialog = false;
        },
        _ => {},
    }
}
