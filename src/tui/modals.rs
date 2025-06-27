use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;

// Quick Add Modal handler
pub async fn handle_quick_add_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_quick_add_modal();
        },
        KeyCode::Enter => {
            let input = app.get_quick_add_input().to_string();
            if !input.trim().is_empty() {
                debug_log(&format!("Creating task with input: '{}'", input));
                app.hide_quick_add_modal();
                // Get default project ID
                let default_project_id = std::env::var("VIKUNJA_DEFAULT_PROJECT")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse::<u64>()
                    .unwrap_or(1);
                debug_log(&format!("Using project ID: {}", default_project_id));
                debug_log("Calling create_task_with_magic...");
                // Create task using API client
                let api_client_guard = api_client.lock().await;
                match api_client_guard.create_task_with_magic(&input, default_project_id).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        // Refresh tasks list
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                    }
                    Err(e) => {
                        debug_log(&format!("ERROR: Failed to create task: {}", e));
                    }
                }
            } else {
                debug_log("Empty input, not creating task");
            }
        },
        KeyCode::Backspace => {
            app.delete_char_from_quick_add();
        },
        KeyCode::Left => {
            app.move_cursor_left();
        },
        KeyCode::Right => {
            app.move_cursor_right();
        },
        KeyCode::Char(c) => {
            app.add_char_to_quick_add(c);
        },
        _ => {},
    }
}

// Edit Modal handler
pub async fn handle_edit_modal(
    app: &mut App,
    key: &KeyEvent,
    api_client: &Arc<Mutex<VikunjaClient>>,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    use crossterm::event::KeyCode;
    match key.code {
        KeyCode::Esc => {
            app.hide_edit_modal();
        },
        KeyCode::Enter => {
            let input = app.get_edit_input().to_string();
            let task_id = app.editing_task_id;
            if !input.trim().is_empty() && task_id.is_some() {
                debug_log(&format!("Updating task ID {} with input: '{}'", task_id.unwrap(), input));
                app.hide_edit_modal();
                // Update task using API client
                let api_client_guard = api_client.lock().await;
                match api_client_guard.update_task_with_magic(task_id.unwrap(), &input).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task updated successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        // Refresh tasks list
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        debug_log(&format!("Tasks refreshed. Total tasks: {}", app.tasks.len()));
                    }
                    Err(e) => {
                        debug_log(&format!("ERROR: Failed to update task: {}", e));
                    }
                }
            } else {
                debug_log("Empty input or no task selected, not updating task");
            }
        },
        KeyCode::Backspace => {
            app.delete_char_from_edit();
        },
        KeyCode::Left => {
            app.move_edit_cursor_left();
        },
        KeyCode::Right => {
            app.move_edit_cursor_right();
        },
        KeyCode::Char(c) => {
            app.add_char_to_edit(c);
        },
        _ => {},
    }
}
