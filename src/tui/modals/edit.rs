use crate::tui::app::App;
use crossterm::event::KeyEvent;
use crate::vikunja_client::VikunjaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::debug::debug_log;

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
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.edit_cursor_position;
                let input = app.get_edit_input();
                let mut new_input = input[..cursor].to_string();
                if let Some(pos) = new_input.rfind(|c| c == '*' || c == '+') {
                    new_input.truncate(pos + 1);
                    new_input.push_str(&suggestion);
                    new_input.push(' ');
                    new_input.push_str(&input[cursor..]);
                    app.edit_input = new_input;
                    app.edit_cursor_position = pos + 1 + suggestion.len() + 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
                return;
            }
            let input = app.get_edit_input().to_string();
            let task_id = app.editing_task_id;
            if !input.trim().is_empty() && task_id.is_some() {
                debug_log(&format!("Updating task ID {} with input: '{}'", task_id.unwrap(), input));
                app.hide_edit_modal();
                let api_client_guard = api_client.lock().await;
                match api_client_guard.update_task_with_magic(task_id.unwrap(), &input).await {
                    Ok(task) => {
                        debug_log(&format!("SUCCESS: Task updated successfully! ID: {:?}, Title: '{}'", task.id, task.title));
                        app.flash_task_id = task.id.map(|id| id as i64);
                        app.flash_start = Some(std::time::Instant::now());
                        drop(api_client_guard);
                        let (tasks, project_map, project_colors) = client_clone.lock().await.get_tasks_with_projects().await.unwrap_or_default();
                        app.all_tasks = tasks;
                        app.project_map = project_map;
                        app.project_colors = project_colors;
                        app.apply_task_filter();
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
        KeyCode::Tab => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                let suggestion = app.suggestions[app.selected_suggestion].clone();
                let cursor = app.edit_cursor_position;
                let input = app.get_edit_input();
                let mut new_input = input[..cursor].to_string();
                if let Some(pos) = new_input.rfind(|c| c == '*' || c == '+') {
                    new_input.truncate(pos + 1);
                    new_input.push_str(&suggestion);
                    new_input.push(' ');
                    new_input.push_str(&input[cursor..]);
                    app.edit_input = new_input;
                    app.edit_cursor_position = pos + 1 + suggestion.len() + 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Down => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                app.selected_suggestion = (app.selected_suggestion + 1) % app.suggestions.len();
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Up => {
            if app.suggestion_mode.is_some() && !app.suggestions.is_empty() {
                if app.selected_suggestion == 0 {
                    app.selected_suggestion = app.suggestions.len() - 1;
                } else {
                    app.selected_suggestion -= 1;
                }
                let input = app.edit_input.clone();
                let cursor = app.edit_cursor_position;
                app.update_suggestions(&input, cursor);
            }
        },
        KeyCode::Backspace => {
            app.delete_char_from_edit();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Left => {
            app.move_edit_cursor_left();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Right => {
            app.move_edit_cursor_right();
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        KeyCode::Char(c) => {
            app.add_char_to_edit(c);
            let input = app.edit_input.clone();
            let cursor = app.edit_cursor_position;
            app.update_suggestions(&input, cursor);
        },
        _ => {},
    }
}
