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
                } else if app_guard.show_label_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::label::handle_label_picker(&mut *app_guard, &key);
                    continue;
                } else if app_guard.show_project_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::project::handle_project_picker(&mut *app_guard, &key);
                    continue;
                } else if app_guard.show_filter_picker {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    crate::tui::pickers::filter::handle_filter_picker(&mut *app_guard, &key, &client_clone).await;
                    continue;
                } else if app_guard.show_attachment_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    if let Some(ref mut modal) = app_guard.attachment_modal {
                        let action = modal.handle_key(match key.code {
                            crossterm::event::KeyCode::Char(c) => c,
                            _ => '\0',
                        });
                        match action {
                            crate::tui::modals::AttachmentModalAction::Close => {
                                app_guard.hide_attachment_modal();
                            }
                            crate::tui::modals::AttachmentModalAction::Download(attachment) => {
                                // Handle download asynchronously
                                let attachment_clone = attachment.clone();
                                let client_clone = client_clone.clone();
                                let app_clone = app.clone();
                                tokio::spawn(async move {
                                    // Set operation in progress
                                    {
                                        let mut app_guard = app_clone.lock().await;
                                        if let Some(ref mut modal) = app_guard.attachment_modal {
                                            modal.operation_in_progress = true;
                                            modal.operation_message = "Downloading attachment...".to_string();
                                        }
                                    }
                                    
                                    // Perform download
                                    // Determine download directory (prefer 'Downloads' or 'downloads')
                                    let download_dir = if std::path::Path::new("Downloads").exists() {
                                        "Downloads"
                                    } else if std::path::Path::new("downloads").exists() {
                                        "downloads"
                                    } else {
                                        // Create default 'downloads' dir
                                        let _ = tokio::fs::create_dir_all("downloads").await;
                                        "downloads"
                                    };
                                    // Determine filename from attachment metadata
                                    let file_name = attachment_clone.file
                                        .as_ref()
                                        .and_then(|f| f.name.clone())
                                        .unwrap_or_else(|| format!("attachment-{}.bin", attachment_clone.id));
                                    let out_path = std::path::Path::new(download_dir).join(&file_name);
                                    let download_result = {
                                        let client = client_clone.lock().await;
                                        client.download_attachment(&attachment_clone, &out_path).await
                                    };
                                    
                                    // Update UI with result
                                    {
                                        let mut app_guard = app_clone.lock().await;
                                        if let Some(ref mut modal) = app_guard.attachment_modal {
                                            modal.operation_in_progress = false;
                                            match download_result {
                                                Ok(_) => {
                                                    modal.operation_message = "Download completed successfully!".to_string();
                                                    app_guard.show_toast("Attachment downloaded successfully!".to_string());
                                                }
                                                Err(e) => {
                                                    modal.operation_message = format!("Download failed: {}", e);
                                                    app_guard.show_toast(format!("Download failed: {}", e));
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                            crate::tui::modals::AttachmentModalAction::Remove(attachment) => {
                                // Handle remove asynchronously
                                // Capture task_id for removal API call
                                let task_id = modal.task_id;
                                let attachment_clone = attachment.clone();
                                let client_clone = client_clone.clone();
                                let app_clone = app.clone();
                                tokio::spawn(async move {
                                    // Set operation in progress
                                    {
                                        let mut app_guard = app_clone.lock().await;
                                        if let Some(ref mut modal) = app_guard.attachment_modal {
                                            modal.operation_in_progress = true;
                                            modal.operation_message = "Removing attachment...".to_string();
                                        }
                                    }
                                    
                                    // Perform remove
                                    let remove_result = {
                                        let client = client_clone.lock().await;
                                        client.remove_attachment(task_id, attachment_clone.id).await
                                    };
                                    
                                    // Update UI with result
                                    {
                                        let mut app_guard = app_clone.lock().await;
                                        if let Some(ref mut modal) = app_guard.attachment_modal {
                                            modal.operation_in_progress = false;
                                            match remove_result {
                                                Ok(_) => {
                                                    modal.operation_message = "Attachment removed successfully!".to_string();
                                                    let task_id = modal.task_id;
                                                    let _ = modal;
                                                    app_guard.show_toast("Attachment removed successfully!".to_string());
                                                    drop(app_guard);
                                                    
                                                    // Refresh attachments
                                                    let refresh_result = {
                                                        let client = client_clone.lock().await;
                                                        client.get_task_attachments(task_id).await
                                                    };
                                                    if let Ok(attachments) = refresh_result {
                                                        let mut app_guard = app_clone.lock().await;
                                                        if let Some(ref mut modal) = app_guard.attachment_modal {
                                                            modal.viewer.attachments = attachments;
                                                            // Ensure selected_index is within bounds
                                                            modal.viewer.selected_index = modal.viewer.selected_index.min(modal.viewer.attachments.len().saturating_sub(1));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    modal.operation_message = format!("Remove failed: {}", e);
                                                    app_guard.show_toast(format!("Remove failed: {}", e));
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                            crate::tui::modals::AttachmentModalAction::Upload => {
                                app_guard.add_debug_message("Upload attachment requested".to_string());
                                app_guard.hide_attachment_modal();
                                app_guard.show_file_picker_modal();
                            }
                            crate::tui::modals::AttachmentModalAction::None => {}
                        }
                    }
                    continue;
                } else if app_guard.show_comments_modal {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    if let Some(ref mut modal) = app_guard.comments_modal {
                        let action = modal.handle_key(&key);
                        match action {
                            crate::tui::modals::CommentsModalAction::Close => {
                                app_guard.hide_comments_modal();
                            }
                            crate::tui::modals::CommentsModalAction::Submit(comment_text) => {
                                if !comment_text.trim().is_empty() {
                                    let task_id = modal.task_id as u64;
                                    let client_clone = client_clone.clone();
                                    let app_clone = app.clone();
                                    let comment_clone = comment_text.clone();
                                    
                                    // Clear the input immediately
                                    modal.clear_input();
                                    
                                    // Submit comment asynchronously
                                    tokio::spawn(async move {
                                        let submit_result = {
                                            let client = client_clone.lock().await;
                                            client.add_comment_to_task(task_id, &comment_clone).await
                                        };
                                        
                                        // Update UI based on result
                                        let mut app_guard = app_clone.lock().await;
                                        match submit_result {
                                            Ok(_) => {
                                                app_guard.add_debug_message("Comment added successfully".to_string());
                                                app_guard.show_toast("Comment added!".to_string());
                                                
                                                // Refresh comments in modal
                                                let client = client_clone.lock().await;
                                                match client.get_comments(task_id).await {
                                                    Ok(comments_list) => {
                                                        // Update modal comments directly
                                                        if let Some(ref mut modal) = app_guard.comments_modal {
                                                            modal.comments = comments_list.clone();
                                                        }
                                                        app_guard.add_debug_message("Comments refreshed in modal".to_string());
                                                    }
                                                    Err(e) => {
                                                        app_guard.add_debug_message(format!("Failed to refresh comments: {}", e));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                app_guard.add_debug_message(format!("Failed to add comment: {}", e));
                                                app_guard.show_toast("Failed to add comment".to_string());
                                            }
                                        }
                                    });
                                }
                            }
                            crate::tui::modals::CommentsModalAction::LoadAttachments(task_id) => {
                                // Handle loading attachments for image preview
                                app_guard.add_debug_message(format!("Loading attachments for task {}", task_id));
                            }
                            crate::tui::modals::CommentsModalAction::ToggleMode => {
                                // Mode toggle is handled internally by the modal
                            }
                            crate::tui::modals::CommentsModalAction::None => {}
                        }
                    }
                    continue;
                // Relations modals - DISABLED: Incomplete feature
                // } else if app_guard.show_relations_modal {
                //     if app_guard.show_add_relation_modal {
                //         drop(app_guard);
                //         let mut app_guard = app.lock().await;
                //         crate::tui::modals::handle_add_relation_modal(&mut *app_guard, &key, &client_clone).await;
                //     } else {
                //         drop(app_guard);
                //         let mut app_guard = app.lock().await;
                //         crate::tui::modals::handle_relations_modal(&mut *app_guard, &key, &client_clone).await;
                //     }
                //     continue;
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
                } else if app_guard.show_subtask_modal {
                    // Handle subtask modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_subtask_modal();
                        }
                        KeyCode::Up => {
                            app_guard.previous_subtask_task();
                        }
                        KeyCode::Down => {
                            app_guard.next_subtask_task();
                        }
                        KeyCode::Enter => {
                            if let Some(ref operation) = app_guard.subtask_operation {
                                match operation {
                                    _ => {
                                        // Handle single selection operations
                                        if let Some((target_task_id, _)) = app_guard.get_selected_subtask_task() {
                                            if let Some(selected_task) = app_guard.get_selected_task() {
                                                let current_task_id = selected_task.id;
                                                let operation = app_guard.subtask_operation.clone();
                                                app_guard.hide_subtask_modal();
                                                
                                                // Handle the subtask operation async
                                                let client = client_clone.lock().await;
                                                match operation {
                                                    Some(crate::tui::app::state::SubtaskOperation::MakeSubtask) => {
                                                        // Make current task a subtask of target task
                                                        match client.create_task_relation(
                                                            current_task_id as u64,
                                                            target_task_id as u64,
                                                            crate::vikunja_client::relations::RelationKind::Subtask
                                                        ).await {
                                                            Ok(_) => {
                                                                app_guard.show_toast("Task made into subtask successfully!".to_string());
                                                                app_guard.add_debug_message(format!("Task {} is now a subtask of {}", current_task_id, target_task_id));
                                                            }
                                                            Err(e) => {
                                                                app_guard.show_toast(format!("Failed to create subtask relation: {}", e));
                                                                app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                            }
                                                        }
                                                    }
                                                    Some(crate::tui::app::state::SubtaskOperation::AddSubtask) => {
                                                        // Make target task a subtask of current task
                                                        match client.create_task_relation(
                                                            target_task_id as u64,
                                                            current_task_id as u64,
                                                            crate::vikunja_client::relations::RelationKind::Subtask
                                                        ).await {
                                                            Ok(_) => {
                                                                app_guard.show_toast("Subtask added successfully!".to_string());
                                                                app_guard.add_debug_message(format!("Task {} is now a subtask of {}", target_task_id, current_task_id));
                                                            }
                                                            Err(e) => {
                                                                app_guard.show_toast(format!("Failed to create subtask relation: {}", e));
                                                                app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            // Toggle selection for bulk operations
                            app_guard.toggle_subtask_task_selection();
                        }
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_subtask_input(c);
                        }
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_subtask_input();
                        }
                        _ => {}
                    }
                    continue;
                } else if app_guard.show_add_subtask_modal {
                    // Handle add subtask modal input
                    match key.code {
                        KeyCode::Esc => {
                            app_guard.hide_add_subtask_modal();
                        }
                        KeyCode::Enter => {
                            // Create new subtask
                            let subtask_title = app_guard.get_add_subtask_input().to_string();
                            if !subtask_title.trim().is_empty() {
                                if let Some(parent_task) = app_guard.get_selected_task() {
                                    let parent_task_id = parent_task.id;
                                    let parent_project_id = parent_task.project_id;
                                    app_guard.hide_add_subtask_modal();
                                    
                                    // Create the subtask
                                    let client = client_clone.lock().await;
                                    let subtask = crate::vikunja_client::VikunjaTask {
                                        id: None,
                                        title: subtask_title.clone(),
                                        description: None,
                                        done: Some(false),
                                        priority: None,
                                        due_date: None,
                                        start_date: None,
                                        project_id: parent_project_id as u64,
                                        labels: None,
                                        assignees: None,
                                        is_favorite: Some(false),
                                    };
                                    
                                    match client.create_task(&subtask).await {
                                        Ok(new_task) => {
                                            app_guard.add_debug_message(format!("Created new task: {}", new_task.title));
                                            
                                            // Create the subtask relation (make new task a subtask of selected task)
                                            if let Some(new_task_id) = new_task.id {
                                                match client.create_task_relation(
                                                    parent_task_id as u64,
                                                    new_task_id,
                                                    crate::vikunja_client::relations::RelationKind::Subtask
                                                ).await {
                                                    Ok(_) => {
                                                        app_guard.show_toast(format!("Created subtask: {}", subtask_title));
                                                        app_guard.add_debug_message(format!("Task {} is now a subtask of {}", new_task_id, parent_task_id));
                                                        
                                                        // Refresh tasks to show the new subtask
                                                        app_guard.refreshing = true;
                                                    }
                                                    Err(e) => {
                                                        app_guard.show_toast(format!("Created task but failed to make it a subtask: {}", e));
                                                        app_guard.add_debug_message(format!("Error creating subtask relation: {}", e));
                                                    }
                                                }
                                            } else {
                                                app_guard.show_toast("Created task but it has no ID".to_string());
                                            }
                                        }
                                        Err(e) => {
                                            app_guard.show_toast(format!("Failed to create subtask: {}", e));
                                            app_guard.add_debug_message(format!("Error creating task: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Left => {
                            app_guard.move_add_subtask_cursor_left();
                        }
                        KeyCode::Right => {
                            app_guard.move_add_subtask_cursor_right();
                        }
                        KeyCode::Home => {
                            app_guard.add_subtask_cursor_position = 0;
                        }
                        KeyCode::End => {
                            app_guard.add_subtask_cursor_position = app_guard.add_subtask_input.len();
                        }
                        KeyCode::Char(c) => {
                            app_guard.add_char_to_add_subtask(c);
                        }
                        KeyCode::Backspace => {
                            app_guard.delete_char_from_add_subtask();
                        }
                        _ => {}
                    }
                    continue;
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
                // handle async task completion toggle
                if key.code == KeyCode::Char('d') {
                    drop(app_guard);
                    let mut app_guard = app.lock().await;
                    if let Some(task_id) = app_guard.toggle_task_completion() {
                        // Sync with API
                        let client = client_clone.lock().await;
                        if let Some(task) = app_guard.tasks.iter().find(|t| t.id == task_id) {
                            let api_task = crate::vikunja_client::VikunjaTask {
                                id: Some(task.id as u64),
                                title: task.title.clone(),
                                description: task.description.clone(),
                                done: Some(task.done),
                                priority: task.priority.map(|p| p as u8),
                                due_date: task.due_date,
                                project_id: task.project_id as u64,
                                labels: None,
                                assignees: None,
                                is_favorite: Some(task.is_favorite),
                                start_date: task.start_date,
                            };
                            match client.update_task(&api_task).await {
                                Ok(_) => {
                                    app_guard.add_debug_message(format!("Task completion synced to API for task {}", task_id));
                                },
                                Err(e) => {
                                    app_guard.add_debug_message(format!("Failed to sync task completion to API: {}", e));
                                    app_guard.show_toast(format!("Sync failed: {}", e));
                                }
                            }
                        }
                    }
                    continue;
                }
                
                // Handle file picker modal
                if app_guard.show_file_picker_modal {
                    if let Some(ref mut modal) = app_guard.file_picker_modal {
                        // Refresh entries if needed
                        if modal.entries.is_empty() {
                            if let Err(e) = modal.refresh_entries().await {
                                app_guard.add_debug_message(format!("Failed to refresh file picker: {}", e));
                                app_guard.hide_file_picker_modal();
                                continue;
                            }
                        }
                        
                        // Handle key events
                        let action = match key.code {
                            crossterm::event::KeyCode::Char(c) => modal.handle_key(c),
                            crossterm::event::KeyCode::Enter => modal.handle_enter(),
                            crossterm::event::KeyCode::Up => {
                                if modal.selected_index > 0 {
                                    modal.selected_index -= 1;
                                }
                                crate::tui::modals::FilePickerAction::None
                            }
                            crossterm::event::KeyCode::Down => {
                                if modal.selected_index < modal.entries.len().saturating_sub(1) {
                                    modal.selected_index += 1;
                                }
                                crate::tui::modals::FilePickerAction::None
                            }
                            _ => crate::tui::modals::FilePickerAction::None,
                        };
                        
                        match action {
                            crate::tui::modals::FilePickerAction::Select(file_path) => {
                                // Handle file selection for upload
                                let file_path_clone = file_path.clone();
                                let client_clone = client_clone.clone();
                                let app_clone = app.clone();
                                // Get task_id from the selected task
                                let task_id = if let Some(task) = app_guard.get_selected_task() {
                                    task.id
                                } else {
                                    // Fallback - we need a task ID
                                    app_guard.hide_file_picker_modal();
                                    app_guard.show_toast("No task selected for upload".to_string());
                                    continue;
                                };
                                
                                app_guard.hide_file_picker_modal();
                                app_guard.show_toast(format!("Uploading {}...", file_path.file_name().unwrap_or_default().to_string_lossy()));
                                
                                tokio::spawn(async move {
                                    // Perform upload
                                    let upload_result = {
                                        let client = client_clone.lock().await;
                                        client.upload_attachment(task_id, &file_path_clone).await
                                    };
                                    
                                    // Update UI with result
                                    {
                                        let mut app_guard = app_clone.lock().await;
                                        match upload_result {
                                            Ok(attachment) => {
                                                app_guard.add_debug_message(format!("Upload successful: attachment ID {}", attachment.id));
                                                app_guard.show_toast("File uploaded successfully!".to_string());
                                                // Refresh attachments if attachment modal is open
                                                if app_guard.show_attachment_modal {
                                                    if let Some(ref modal) = app_guard.attachment_modal {
                                                        let task_id = modal.task_id;
                                                        let refresh_result = {
                                                            let client = client_clone.lock().await;
                                                            client.get_task_attachments(task_id).await
                                                        };
                                                        match refresh_result {
                                                            Ok(attachments) => {
                                                                app_guard.add_debug_message(format!("Refreshed {} attachments", attachments.len()));
                                                                if let Some(ref mut modal) = app_guard.attachment_modal {
                                                                    modal.viewer.attachments = attachments;
                                                                }
                                                            }
                                                            Err(e) => {
                                                                app_guard.add_debug_message(format!("Failed to refresh attachments: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                let error_msg = format!("Upload failed: {}", e);
                                                app_guard.add_debug_message(error_msg.clone());
                                                app_guard.show_toast(error_msg);
                                            }
                                        }
                                    }
                                });
                            }
                            crate::tui::modals::FilePickerAction::Navigate(new_path) => {
                                modal.current_path = new_path;
                                modal.selected_index = 0;
                                modal.entries.clear();                  // reload entries immediately
                                modal.refresh_entries_sync();
                            }
                            crate::tui::modals::FilePickerAction::ToggleHidden => {
                                modal.show_hidden = !modal.show_hidden;
                                modal.entries.clear();                  // reload entries immediately
                                modal.refresh_entries_sync();
                            }
                            crate::tui::modals::FilePickerAction::Cancel => {
                                app_guard.hide_file_picker_modal();
                            }
                            crate::tui::modals::FilePickerAction::None => {}
                        }
                    }
                    continue;
                }

                // Handle URL modal
                if app_guard.show_url_modal {
                    if let Some(ref mut modal) = app_guard.url_modal {
                        let action = match key.code {
                            crossterm::event::KeyCode::Char(c) => modal.handle_key(c),
                            crossterm::event::KeyCode::Enter => modal.handle_enter(),
                            crossterm::event::KeyCode::Up => {
                                modal.handle_up();
                                crate::tui::modals::UrlModalAction::None
                            }
                            crossterm::event::KeyCode::Down => {
                                modal.handle_down();
                                crate::tui::modals::UrlModalAction::None
                            }
                            crossterm::event::KeyCode::Esc => crate::tui::modals::UrlModalAction::Cancel,
                            _ => crate::tui::modals::UrlModalAction::None,
                        };
                        
                        match action {
                            crate::tui::modals::UrlModalAction::OpenUrl(url) => {
                                app_guard.hide_url_modal();
                                // Open URL in background to avoid blocking UI
                                let url_clone = url.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = crate::url_utils::open_url(&url_clone) {
                                        eprintln!("Failed to open URL {}: {}", url_clone, e);
                                    }
                                });
                                app_guard.show_toast(format!("Opening: {}", url));
                            }
                            crate::tui::modals::UrlModalAction::Cancel => {
                                app_guard.hide_url_modal();
                            }
                            crate::tui::modals::UrlModalAction::None => {}
                        }
                    }
                    continue;
                }
                
                // handle dispatch_key and refresh
                let key_handled = dispatch_key(&mut *app_guard, key);
                
                // After any navigation key, check if we need to fetch detailed task data
                if key_handled && (key.code == KeyCode::Up || key.code == KeyCode::Down || 
                                  key.code == KeyCode::Char('j') || key.code == KeyCode::Char('k') ||
                                  key.code == KeyCode::Char('g') || key.code == KeyCode::Char('G')) {
                    if let Some(task) = app_guard.get_selected_task() {
                        let task_id = task.id;
                        if !app_guard.detailed_task_cache.contains_key(&task_id) {
                            let client_clone = client_clone.clone();
                            let app_clone = app.clone();
                            tokio::spawn(async move {
                                let client = client_clone.lock().await;
                                if let Ok(detailed_task) = client.get_task_detailed(task_id as u64).await {
                                    let mut app_guard = app_clone.lock().await;
                                    app_guard.cache_detailed_task(detailed_task);
                                }
                            });
                        }
                    }
                }
                
                if key_handled {
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
        KeyCode::Up => {
            if app.show_advanced_features_modal {
                if app.selected_advanced_feature_index > 0 {
                    app.selected_advanced_feature_index -= 1;
                }
                true
            } else {
                app.previous_task();
                true
            }
        }
        KeyCode::Down => {
            if app.show_advanced_features_modal {
                let max_index = 5; // Number of advanced features - 1
                if app.selected_advanced_feature_index < max_index {
                    app.selected_advanced_feature_index += 1;
                }
                true
            } else {
                app.next_task();
                true
            }
        }
        Char('g') => { app.jump_to_top(); true }
        Char('G') => { app.jump_to_bottom(); true }
        Char('?') => { app.show_help_modal(); true }
        Char('q') => {
            if app.show_advanced_features_modal {
                app.hide_advanced_features_modal();
                true
            } else {
                // Prompt for quit confirmation
                app.confirm_quit();
                true
            }
        }
        Char('i') => { app.toggle_info_pane(); true }
        Char('x') => { app.toggle_debug_pane(); true }
        // Navigation: move selection down/up
        Char('j') => { 
            if app.show_advanced_features_modal {
                let max_index = 5; // Number of advanced features - 1
                if app.selected_advanced_feature_index < max_index {
                    app.selected_advanced_feature_index += 1;
                }
                true
            } else {
                app.next_task(); 
                true 
            }
        }
        Char('k') => { 
            if app.show_advanced_features_modal {
                if app.selected_advanced_feature_index > 0 {
                    app.selected_advanced_feature_index -= 1;
                }
                true
            } else {
                app.previous_task(); 
                true 
            }
        }
        // Switch layouts backward/forward
        Char('l') => { app.switch_to_next_layout(); true }
        // Cycle filters backward/forward
        Char('H') => { app.cycle_task_filter(); true }
        Char('L') => { app.cycle_task_filter(); true }
        // Advanced features modal disabled - feature not ready
        // Char('.') => { app.show_advanced_features_modal(); true }
        Char('E') => { app.hide_help_modal(); app.show_form_edit_modal(); true }
        Char('e') => { app.show_edit_modal(); true }
        Char('o') => {
            // Open URLs from the selected task
            crate::debug::debug_log("User pressed 'o' - attempting to open URLs from selected task");
            if let Some(basic_task) = app.get_selected_task() {
                // Try to get the detailed task with comments first, fall back to basic task
                let task_to_use = app.get_detailed_task(basic_task.id).unwrap_or(basic_task);
                crate::debug::debug_log(&format!("Selected task: id={}, title={:?}, has_comments={}, using_detailed_cache={}", 
                    task_to_use.id, task_to_use.title, task_to_use.comments.is_some(), 
                    app.get_detailed_task(basic_task.id).is_some()));
                let urls = crate::url_utils::extract_urls_from_task(task_to_use);
                crate::debug::debug_log(&format!("extract_urls_from_task returned {} URLs", urls.len()));
                if !urls.is_empty() {
                    app.show_url_modal(urls);
                } else {
                    app.show_toast("No URLs found in this task".to_string());
                }
            } else {
                crate::debug::debug_log("No task selected");
            }
            true
        }
        Char('p') => { app.show_project_picker(); true }
        Char('f') => { app.show_filter_picker(); true }
        Char(' ') => { app.show_quick_actions_modal(); true }
        Char('a') => { 
            if app.show_advanced_features_modal {
                // Direct activation of attachment management
                app.hide_advanced_features_modal();
                app.show_attachment_modal();
                true
            } else {
                app.show_quick_add_modal(); 
                true 
            }
        }
        Char('c') => {
            if app.show_advanced_features_modal {
                app.hide_advanced_features_modal();
                app.show_comments_modal();
                true
            } else {
                false
            }
        }
        Char('r') => { 
            if app.show_advanced_features_modal {
                // Direct activation of task relations
                app.hide_advanced_features_modal();
                app.add_debug_message("Task relations feature requested".to_string());
                app.show_toast("Task relations feature coming soon!".to_string());
                true
            } else {
                false
            }
        }
        Char('h') => { 
            if app.show_advanced_features_modal {
                // Direct activation of task history
                app.hide_advanced_features_modal();
                app.add_debug_message("Task history feature requested".to_string());
                app.show_toast("Task history feature coming soon!".to_string());
                true
            } else {
                app.switch_to_previous_layout(); 
                true 
            }
        }
        Char('s') => { 
            if app.show_advanced_features_modal {
                app.hide_advanced_features_modal();
                app.add_debug_message("Subtasks feature requested".to_string());
                app.show_toast("Subtasks feature coming soon!".to_string());
                true
            } else {
                /* async star toggle handled in event loop */ 
                true 
            }
        }
        Char('t') => { 
            if app.show_advanced_features_modal {
                // Direct activation of time tracking
                app.hide_advanced_features_modal();
                app.add_debug_message("Time tracking feature requested".to_string());
                app.show_toast("Time tracking feature coming soon!".to_string());
                true
            } else {
                false
            }
        }
        Enter => {
            if app.show_confirmation_dialog {
                // handled async in event loop
                true
            } else if app.show_advanced_features_modal {
                // Handle advanced feature selection
                match app.selected_advanced_feature_index {
                    0 => { // Attachment Management
                        app.hide_advanced_features_modal();
                        app.show_attachment_modal();
                    }
                    1 => { // Comments
                        app.hide_advanced_features_modal();
                        app.show_comments_modal();
                    }
                    2 => { // Task Relations
                        app.hide_advanced_features_modal();
                        app.add_debug_message("Task relations feature requested".to_string());
                    }
                    3 => { // Task History
                        app.hide_advanced_features_modal();
                        app.add_debug_message("Task history feature requested".to_string());
                    }
                    4 => { // Subtasks
                        app.hide_advanced_features_modal();
                        if app.get_selected_task().is_some() {
                            app.show_subtask_modal(crate::tui::app::state::SubtaskOperation::AddSubtask);
                        } else {
                            app.show_toast("Select a task first".to_string());
                        }
                    }
                    5 => { // Time Tracking
                        app.hide_advanced_features_modal();
                        app.add_debug_message("Time tracking feature requested".to_string());
                    }
                    _ => {
                        app.hide_advanced_features_modal();
                    }
                }
                true
            } else {
                true
            }
        }
        Char('n') => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            }
            true
        }
        // Advanced features modal navigation
        Up => {
            if app.show_advanced_features_modal {
                if app.selected_advanced_feature_index > 0 {
                    app.selected_advanced_feature_index -= 1;
                }
                true
            } else {
                false
            }
        }
        Down => {
            if app.show_advanced_features_modal {
                let max_index = 5; // Number of advanced features - 1
                if app.selected_advanced_feature_index < max_index {
                    app.selected_advanced_feature_index += 1;
                }
                true
            } else {
                false
            }
        }
        Esc => {
            if app.show_confirmation_dialog {
                app.cancel_confirmation();
            } else if app.show_advanced_features_modal {
                app.hide_advanced_features_modal();
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
    
    // Save current filter state before refresh
    let current_filter_id = app.current_filter_id;
    let current_project_id = app.current_project_id;
    let current_task_filter = app.task_filter.clone();
    let active_project_override = app.active_project_override.clone();
    
    let client = client.lock().await;
    match client.get_tasks_with_projects().await {
        Ok((tasks, project_map, project_colors)) => {
            app.all_tasks = tasks;
            app.project_map = project_map;
            app.project_colors = project_colors;
            
            // Reapply the current filter state after refresh
            if let Some(filter_id) = current_filter_id {
                // If a saved filter was active, reapply it
                app.current_filter_id = Some(filter_id);
                app.active_project_override = active_project_override;
                
                // Fetch tasks for the filter
                match client.get_tasks_for_filter(filter_id).await {
                    Ok(filter_tasks) => {
                        app.apply_filter_tasks(filter_tasks);
                        app.show_toast("Refreshed with filter applied!".to_string());
                    },
                    Err(e) => {
                        app.add_debug_message(format!("Failed to fetch filter tasks after refresh: {}", e));
                        // Fall back to applying task filter to all tasks
                        app.apply_task_filter();
                        app.show_toast("Refreshed! (Filter fetch failed)".to_string());
                    }
                }
            } else if current_project_id.is_some() {
                // If a project was selected, reapply project filter
                app.current_project_id = current_project_id;
                app.task_filter = current_task_filter;
                app.apply_project_filter();
                app.show_toast("Refreshed with project filter applied!".to_string());
            } else {
                // If no special filter was active, just apply the task filter
                app.task_filter = current_task_filter;
                app.apply_task_filter();
                app.show_toast("Refreshed!".to_string());
            }
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
