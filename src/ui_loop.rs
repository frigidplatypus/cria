use std::sync::Arc;
use tokio::sync::Mutex;
use crossterm::event::{KeyEvent, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use crate::tui::app::state::App;
use crate::tui::events::EventHandler;
use crate::tui::ui::main::draw;
use crate::vikunja_client::VikunjaClient;
// dispatch_key and refresh_from_api moved here from main.rs
use crate::tui::modals::{handle_quick_add_modal, handle_edit_modal, handle_form_edit_modal};

/// Run the main UI event loop
pub async fn run_ui(
                // (removed misplaced async star toggle from function signature)
    app: Arc<Mutex<App>>,
    client_clone: Arc<Mutex<VikunjaClient>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let event_handler = EventHandler::new(250);

    loop {
        {
            let app_guard = app.lock().await;
            terminal.draw(|f| draw(f, &app_guard))?;
        }

        match event_handler.next()? {
            // Handle key events only on Press or Repeat, ignore Release
            crate::tui::events::Event::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
                let mut app_guard = app.lock().await;

                // Modal input handling
                if app_guard.show_quick_add_modal {
                    // Route key to quick add modal handler
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_quick_add_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_edit_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_edit_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_form_edit_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    handle_form_edit_modal(&mut *app_guard, &key, &client_clone, &client_clone).await;
                    continue;
                } else if app_guard.show_quick_actions_modal {
                    // Handle quick actions modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_quick_actions_modal();
                        }
                        KeyCode::Up => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if !quick_actions.is_empty() && app_guard.selected_quick_action_index as usize > 0 {
                                    app_guard.selected_quick_action_index -= 1;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if !quick_actions.is_empty() && (app_guard.selected_quick_action_index as usize + 1) < quick_actions.len() {
                                    app_guard.selected_quick_action_index += 1;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if (app_guard.selected_quick_action_index as usize) < quick_actions.len() {
                                    let action = quick_actions[app_guard.selected_quick_action_index as usize].clone();
                                    app_guard.hide_quick_actions_modal();
                                    apply_quick_action_and_sync(&mut *app_guard, action, &client_clone).await;
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            // Direct character-based quick actions
                            if let Some(ref quick_actions) = app_guard.config.quick_actions {
                                if let Some((idx, action)) = quick_actions.iter().enumerate().find(|(_, a)| a.key == c.to_string()) {
                                    let action = action.clone();
                                    app_guard.hide_quick_actions_modal();
                                    app_guard.selected_quick_action_index = idx;
                                    apply_quick_action_and_sync(&mut *app_guard, action, &client_clone).await;
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                } else if app_guard.quick_action_mode {
                    // Handle quick action mode (activated with '.')
                    if app_guard.is_quick_action_mode_expired() {
                        app_guard.exit_quick_action_mode();
                    } else {
                        match key.code {
                            KeyCode::Char(' ') => {
                                app_guard.exit_quick_action_mode();
                            }
                            KeyCode::Esc => {
                                app_guard.exit_quick_action_mode();
                            }
                            KeyCode::Char(c) => {
                                if let Some(action) = app_guard.config.get_quick_action(&c.to_string()) {
                                    let action = action.clone();
                                    app_guard.exit_quick_action_mode();
                                    apply_quick_action_and_sync(&mut *app_guard, action, &client_clone).await;
                                } else {
                                    app_guard.exit_quick_action_mode();
                                    app_guard.add_debug_message(format!("No quick action configured for key: {}", c));
                                }
                            }
                            _ => {
                                app_guard.exit_quick_action_mode();
                            }
                        }
                        continue;
                    }
                }

                // Handle Ctrl key combinations first
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('z') => {
                            // Ctrl+Z - Undo
                            if let Some(task_id) = app_guard.undo_last_action() {
                                // Update the corresponding task in all_tasks
                                let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                if let Some(updated_task) = updated_task {
                                    if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                        *task = updated_task;
                                    }
                                }
                                // Show visual feedback
                                app_guard.add_debug_message("Undo operation completed".to_string());
                            } else {
                                app_guard.add_debug_message("Nothing to undo".to_string());
                            }
                            continue;
                        },
                        KeyCode::Char('y') => {
                            // Ctrl+Y - Redo
                            if let Some(task_id) = app_guard.redo_last_action() {
                                // Update the corresponding task in all_tasks
                                let updated_task = app_guard.tasks.iter().find(|t| t.id == task_id).cloned();
                                if let Some(updated_task) = updated_task {
                                    if let Some(task) = app_guard.all_tasks.iter_mut().find(|t| t.id == task_id) {
                                        *task = updated_task;
                                    }
                                }
                                // Show visual feedback
                                app_guard.add_debug_message("Redo operation completed".to_string());
                            } else {
                                app_guard.add_debug_message("Nothing to redo".to_string());
                            }
                            continue;
                        },
                        _ => {}
                    }
                    continue; // Skip the regular key handling for Ctrl combinations
                }

                // Handle confirmation dialog actions async (Enter/y)
                if app_guard.show_confirmation_dialog && (key.code == KeyCode::Enter || (matches!(key.code, KeyCode::Char('y')))) {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    let client = client_clone.lock().await;
                    app_guard.confirm_action_async(&*client).await;
                    continue;
                }

                // handle async star toggle
                if key.code == KeyCode::Char('s') {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    let client = client_clone.lock().await;
                    app_guard.toggle_star_selected_task_async(&*client).await;
                    continue;
                }
                // handle dispatch_key and refresh
                if dispatch_key(&mut *app_guard, key) {
                    continue;
                }
                if key.code == KeyCode::Char('r') {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    refresh_from_api(&mut *app_guard, &client_clone).await;
                    continue;
                }

                // other modal and Ctrl/quick-action handling...
                // TODO: move remaining branches here
            }
            crate::tui::events::Event::Tick => {
                let app_guard = app.lock().await;
                // TODO: clear expired notifications / flash
                terminal.draw(|f| draw(f, &app_guard))?;
            }
            // Ignore other events
            _ => {}
        }

        // exit on quit
        if !app.lock().await.running {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// Handle key events, return true if event was handled
fn dispatch_key(app: &mut App, key: KeyEvent) -> bool {
    use KeyCode::*;
    match key.code {
        Char('?') => { app.show_help_modal(); true }
        Char('q') => {
            // Prompt for quit confirmation
            app.confirm_quit();
            true
        }
        Char('s') => { /* async star toggle handled in event loop */ true }
        Char('i') => { app.toggle_info_pane(); true }
        // Navigation: move selection down/up
        Char('j') | Down => { app.next_task(); true }
        Char('k') | Up => { app.previous_task(); true }
        // Cycle filters backward/forward
        Char('h') => { app.cycle_task_filter(); true }
        Char('l') => { app.cycle_task_filter(); true }
        // Quick action mode via dot
        Char('.') => { app.enter_quick_action_mode(); true }
        Char('E') => { app.hide_help_modal(); app.show_form_edit_modal(); true }
        Char('e') => { app.show_edit_modal(); true }
        Char('p') => { app.show_project_picker(); true }
        Char(' ') => { app.show_quick_actions_modal(); true }
        Char('a') => { app.show_quick_add_modal(); true }
        Enter => {
            if app.show_confirmation_dialog {
                // handled async in event loop
            }
            true
        }
        Char('n') => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            }
            true
        }
        Esc => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            } else {
                // Close any open modal or dialog
                app.close_all_modals();
            }
            true
        }
        Char('y') => {
            if app.show_confirmation_dialog {
                // handled async in event loop
            }
            true
        }
        Char('D') => { app.request_delete_task(); true }
        _ => false,
    }
}

/// Refresh tasks from API (stub implementation)
async fn refresh_from_api(
    app: &mut App,
    client: &Arc<Mutex<VikunjaClient>>,
) {
    app.refreshing = true;
    let client = client.lock().await;
    match client.get_tasks_with_projects().await {
        Ok((tasks, project_map, project_colors)) => {
            app.all_tasks = tasks;
            app.project_map = project_map;
            app.project_colors = project_colors;
            // Only show active tasks after refresh
            app.tasks = app.all_tasks.iter().filter(|t| !t.done).cloned().collect();
            app.show_toast("Refreshed!".to_string());
        }
        Err(e) => {
            app.show_toast(format!("Refresh failed: {}", e));
        }
    }
    app.refreshing = false;
}

/// Apply quick action and sync with API (extracted from old main.rs)
async fn apply_quick_action_and_sync(
    app: &mut App,
    action: crate::config::QuickAction,
    client_clone: &Arc<Mutex<VikunjaClient>>,
) {
    match app.apply_quick_action(&action) {
        Ok(_) => {
            app.add_debug_message(format!("Quick action applied: {} -> {}", action.key, action.target));
            
            // Update the task on the server - handle labels differently
            let selected_task = app.get_selected_task().cloned();
            if let Some(task) = selected_task {
                if action.action == "label" {
                    // For label actions, use the specialized label API
                    if let Some(label_id) = app.label_map.iter().find_map(|(id, name)| {
                        if name == &action.target { Some(*id) } else { None }
                    }) {
                        app.add_debug_message(format!("Adding label {} (id={}) to task {}", action.target, label_id, task.id));
                        match client_clone.lock().await.add_label_to_task(task.id as u64, label_id as u64).await {
                            Ok(_) => {
                                app.add_debug_message(format!("Label API update successful for task {}", task.id));
                                app.show_toast(format!("Label added: {}", action.target));
                            },
                            Err(e) => {
                                app.add_debug_message(format!("Label API update failed: {}", e));
                                app.show_toast(format!("Label update failed: {}", e));
                            }
                        }
                    } else {
                        app.add_debug_message(format!("Label '{}' not found in label_map", action.target));
                        app.show_toast(format!("Label '{}' not found", action.target));
                    }
                } else {
                    // For non-label actions, use the general task update
                    let api_task = crate::vikunja_client::VikunjaTask {
                        id: Some(task.id as u64),
                        title: task.title.clone(),
                        description: task.description.clone(),
                        done: Some(task.done),
                        priority: task.priority.map(|p| p as u8),
                        due_date: task.due_date,
                        project_id: task.project_id as u64,
                        labels: None, // Don't update labels via general task update
                        assignees: None,
                        is_favorite: Some(task.is_favorite),
                        start_date: task.start_date,
                    };
                    match client_clone.lock().await.update_task(&api_task).await {
                        Ok(_) => {
                            app.show_toast(format!("Quick action applied: {} -> {}", action.key, action.target));
                        },
                        Err(e) => {
                            app.add_debug_message(format!("API update failed: {}", e));
                            app.show_toast(format!("Update failed: {}", e));
                        }
                    }
                }
                
                // Add visual flash feedback
                if let Some(task) = app.get_selected_task() {
                    app.flash_task_id = Some(task.id);
                    app.flash_start = Some(chrono::Local::now());
                    app.flash_cycle_count = 0;
                    app.flash_cycle_max = 4;
                }
            }
        }
        Err(e) => {
            app.add_debug_message(format!("Quick action error: {}", e));
            app.show_toast(format!("Quick action error: {}", e));
        }
    }
}
