use crossterm::event::{KeyCode, KeyEvent};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::tui::app::state::App;
use crate::vikunja_client::VikunjaClient;

pub async fn handle_relations_modal(
    app: &mut App,
    key: &KeyEvent,
    client: &Arc<Mutex<VikunjaClient>>,
) {
    match key.code {
        KeyCode::Esc => {
            app.hide_relations_modal();
        }
        KeyCode::Char('a') => {
            app.show_add_relation_modal();
        }
        KeyCode::Char('d') => {
            // Delete selected relation (would need selection logic)
            // TODO: Implement relation deletion
        }
        KeyCode::Char('r') => {
            // Refresh relations
            if let Some(task_id) = app.relations_task_id {
                refresh_task_relations(app, task_id, client).await;
            }
        }
        _ => {}
    }
}

pub async fn handle_add_relation_modal(
    app: &mut App,
    key: &KeyEvent,
    client: &Arc<Mutex<VikunjaClient>>,
) {
    match key.code {
        KeyCode::Esc => {
            app.hide_add_relation_modal();
        }
        KeyCode::Enter => {
            if let Some(task_id) = app.relations_task_id {
                create_relation(app, task_id, client).await;
            }
        }
        KeyCode::Tab => {
            app.next_relation_kind();
        }
        KeyCode::BackTab => {
            app.previous_relation_kind();
        }
        KeyCode::Up => {
            app.previous_relation_kind();
        }
        KeyCode::Down => {
            app.next_relation_kind();
        }
        KeyCode::Char(c) => {
            app.add_char_to_relation_input(c);
        }
        KeyCode::Backspace => {
            app.delete_char_from_relation_input();
        }
        KeyCode::Left => {
            app.move_relation_cursor_left();
        }
        KeyCode::Right => {
            app.move_relation_cursor_right();
        }
        _ => {}
    }
}

async fn create_relation(
    app: &mut App,
    task_id: i64,
    client: &Arc<Mutex<VikunjaClient>>,
) {
    let input = app.add_relation_input.trim();
    if input.is_empty() {
        app.show_toast("Please enter a task ID or title".to_string());
        return;
    }

    let relation_kind = match app.get_selected_relation_kind() {
        Some(kind) => kind.clone(),
        None => {
            app.show_toast("No relation type selected".to_string());
            return;
        }
    };

    // Try to parse as task ID first
    let other_task_id = if let Ok(id) = input.parse::<u64>() {
        id
    } else {
        // Search for task by title
        match find_task_by_title(app, input) {
            Some(id) => id as u64,
            None => {
                app.show_toast(format!("Task not found: {}", input));
                return;
            }
        }
    };

    let client_guard = client.lock().await;
    match client_guard.create_task_relation(task_id as u64, other_task_id, relation_kind.clone()).await {
        Ok(_) => {
            app.show_toast(format!("Relation created: {} task {}", relation_kind.display_name(), other_task_id));
            app.hide_add_relation_modal();
            // Refresh the task to get updated relations
            refresh_task_relations(app, task_id, client).await;
        }
        Err(e) => {
            app.show_toast(format!("Failed to create relation: {}", e));
        }
    }
}

async fn refresh_task_relations(
    app: &mut App,
    task_id: i64,
    client: &Arc<Mutex<VikunjaClient>>,
) {
    let client_guard = client.lock().await;
    match client_guard.get_task_relations(task_id as u64).await {
        Ok(relations) => {
            // Update the task in both tasks and all_tasks
            if let Some(task) = app.tasks.iter_mut().find(|t| t.id == task_id) {
                task.related_tasks = Some(relations.clone());
            }
            if let Some(task) = app.all_tasks.iter_mut().find(|t| t.id == task_id) {
                task.related_tasks = Some(relations);
            }
            app.add_debug_message(format!("Refreshed relations for task {}", task_id));
        }
        Err(e) => {
            app.show_toast(format!("Failed to refresh relations: {}", e));
        }
    }
}

fn find_task_by_title(app: &App, title: &str) -> Option<i64> {
    let title_lower = title.to_lowercase();
    
    // First try exact match
    for task in &app.all_tasks {
        if task.title.to_lowercase() == title_lower {
            return Some(task.id);
        }
    }
    
    // Then try partial match
    for task in &app.all_tasks {
        if task.title.to_lowercase().contains(&title_lower) {
            return Some(task.id);
        }
    }
    
    None
}