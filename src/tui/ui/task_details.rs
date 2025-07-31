// Task details pane rendering

use crate::tui::app::state::App;
use ratatui::prelude::*;
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Paragraph, Block, Borders, Wrap};
use ratatui::text::{Line, Span};
use chrono::Datelike;
use super::hex_to_color;


pub fn draw_task_details(f: &mut Frame, app: &App, area: Rect) {
    let selected_task = app.get_selected_task();
    
    let details = if let Some(basic_task) = selected_task {
        // Check if we have detailed task data with comments
        let task = app.get_detailed_task(basic_task.id).unwrap_or(basic_task);
        
        let project_name = app.project_map.get(&(task.project_id as i64))
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        
        let mut details_lines = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&task.title)
            ]),
            Line::from(""),
        ];

        // Description
        if let Some(description) = &task.description {
            if !description.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Description: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(description)
                ]));
                details_lines.push(Line::from(""));
            }
        }

                // Project
        if task.project_id > 0 {
            if let Some(project_name) = app.project_map.get(&task.project_id) {
                let color = app.project_colors.get(&task.project_id)
                    .map(|hex_str| hex_to_color(hex_str))
                    .unwrap_or(Color::Blue);
                
                details_lines.push(Line::from(vec![
                    Span::styled("Project: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(&**project_name, Style::default().fg(color))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Parent/Subtask relationships
        if let Some(ref related_tasks) = task.related_tasks {
            // Check if this task has a parent (is a subtask)
            if let Some(parent_tasks) = related_tasks.get("parenttask") {
                if !parent_tasks.is_empty() {
                    let parent_task = &parent_tasks[0];
                    details_lines.push(Line::from(vec![
                        Span::styled("Parent Task: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("🔗", Style::default().fg(Color::Yellow)),
                        Span::raw(format!(" {}", parent_task.title))
                    ]));
                    details_lines.push(Line::from(""));
                }
            }

            // Check if this task has subtasks (is a parent)
            if let Some(subtasks) = related_tasks.get("subtask") {
                if !subtasks.is_empty() {
                    details_lines.push(Line::from(vec![
                        Span::styled("Subtasks: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("📋", Style::default().fg(Color::Green)),
                        Span::raw(format!(" {} subtask(s)", subtasks.len()))
                    ]));
                    
                    // Show up to 5 subtasks by name
                    for (_i, subtask) in subtasks.iter().take(5).enumerate() {
                        let status_icon = if subtask.done { "✓" } else { "○" };
                        let status_color = if subtask.done { Color::Green } else { Color::Gray };
                        
                        details_lines.push(Line::from(vec![
                            Span::raw("  • "),
                            Span::styled(status_icon, Style::default().fg(status_color)),
                            Span::raw(format!(" {}", subtask.title))
                        ]));
                    }
                    
                    // Show "and X more" if there are more than 5 subtasks
                    if subtasks.len() > 5 {
                        details_lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(format!("... and {} more", subtasks.len() - 5), 
                                Style::default().fg(Color::DarkGray))
                        ]));
                    }
                    
                    details_lines.push(Line::from(""));
                }
            }
        }

        // Task color
        if let Some(hex_color) = &task.hex_color {
            if !hex_color.is_empty() {
                let color = hex_to_color(hex_color);
                details_lines.push(Line::from(vec![
                    Span::styled("Color: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("●", Style::default().fg(color)),
                    Span::raw(format!(" {}", hex_color))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Cover image
        if let Some(cover_id) = task.cover_image_attachment_id {
            if cover_id > 0 {
                details_lines.push(Line::from(vec![
                    Span::styled("Cover Image: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("🖼", Style::default().fg(Color::Green)),
                    Span::raw(format!(" Attachment #{}", cover_id))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Status
        details_lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(if task.done { "Completed" } else { "Pending" })
        ]));
        details_lines.push(Line::from(""));

        // Completion percentage
        if let Some(percent) = task.percent_done {
            if percent > 0 {
                details_lines.push(Line::from(vec![
                    Span::styled("Progress: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{}%", percent))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Priority
        if let Some(priority) = task.priority {
            if priority > 0 {
                let priority_color = match priority {
                    5 => Color::Red,
                    4 => Color::Rgb(255, 165, 0),
                    3 => Color::Yellow,
                    2 => Color::Blue,
                    1 => Color::Magenta,
                    _ => Color::White,
                };
                details_lines.push(Line::from(vec![
                    Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("●", Style::default().fg(priority_color)),
                    Span::raw(format!(" !{}", priority))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Due date
        if let Some(due_date) = &task.due_date {
            if due_date.year() > 1900 {
                details_lines.push(Line::from(vec![
                    Span::styled("Due Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(due_date.format("%Y-%m-%d %H:%M").to_string())
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Start date
        if let Some(start_date) = &task.start_date {
            if start_date.year() > 1900 {
                details_lines.push(Line::from(vec![
                    Span::styled("Start Date: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(start_date.format("%Y-%m-%d %H:%M").to_string())
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // End date
        if let Some(end_date) = &task.end_date {
            if !end_date.is_empty() {
                // Try to parse as DateTime first, then fallback to string display
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(end_date) {
                    if parsed_date.year() > 1900 {
                        details_lines.push(Line::from(vec![
                            Span::styled("End Date: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(parsed_date.format("%Y-%m-%d %H:%M").to_string())
                        ]));
                        details_lines.push(Line::from(""));
                    }
                } else {
                    // If parsing fails, just display the string
                    details_lines.push(Line::from(vec![
                        Span::styled("End Date: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(end_date)
                    ]));
                    details_lines.push(Line::from(""));
                }
            }
        }

        // Done date (when task was completed)
        if task.done {
            if let Some(done_at) = &task.done_at {
                if !done_at.is_empty() {
                    if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(done_at) {
                        if parsed_date.year() > 1900 {
                            details_lines.push(Line::from(vec![
                                Span::styled("Completed: ", Style::default().add_modifier(Modifier::BOLD)),
                                Span::raw(parsed_date.format("%Y-%m-%d %H:%M").to_string())
                            ]));
                            details_lines.push(Line::from(""));
                        }
                    } else {
                        // If parsing fails, just display the string
                        details_lines.push(Line::from(vec![
                            Span::styled("Completed: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(done_at)
                        ]));
                        details_lines.push(Line::from(""));
                    }
                }
            }
        }

        // Starred/Favorite
        if task.is_favorite {
            details_lines.push(Line::from(vec![
                Span::styled("Starred: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled("★", Style::default().fg(Color::Yellow)),
                Span::raw(" Yes")
            ]));
            details_lines.push(Line::from(""));
        }

        // Labels
        if let Some(labels) = &task.labels {
            if !labels.is_empty() {
                let mut labels_line_spans = vec![Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD))];
                for (i, label) in labels.iter().enumerate() {
                    let color = hex_to_color(label.hex_color.as_deref().unwrap_or(""));
                    labels_line_spans.push(Span::styled(&label.title, Style::default().fg(color)));
                    if i < labels.len() - 1 {
                        labels_line_spans.push(Span::raw(", "));
                    }
                }
                details_lines.push(Line::from(labels_line_spans));
                details_lines.push(Line::from(""));
            }
        }

        // Assignees
        if let Some(assignees) = &task.assignees {
            if !assignees.is_empty() {
                let mut assignees_line_spans = vec![Span::styled("Assignees: ", Style::default().add_modifier(Modifier::BOLD))];
                for (i, assignee) in assignees.iter().enumerate() {
                    let display_name = if let Some(name) = &assignee.name {
                        if !name.is_empty() {
                            format!("{} (@{})", name, assignee.username)
                        } else {
                            format!("@{}", assignee.username)
                        }
                    } else {
                        format!("@{}", assignee.username)
                    };
                    assignees_line_spans.push(Span::styled(display_name, Style::default().fg(Color::Cyan)));
                    if i < assignees.len() - 1 {
                        assignees_line_spans.push(Span::raw(", "));
                    }
                }
                details_lines.push(Line::from(assignees_line_spans));
                details_lines.push(Line::from(""));
            }
        }

        // Attachments
        if let Some(attachments) = &task.attachments {
            if !attachments.is_empty() {
                let image_count = attachments.iter()
                    .filter_map(|a| a.file.as_ref())
                    .filter(|f| {
                        let filename = f.name.as_deref().unwrap_or("");
                        let mime = f.mime.as_deref();
                        let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tga"];
                        
                        // Check file extension
                        if let Some(ext) = filename.split('.').last() {
                            if image_extensions.iter().any(|&img_ext| ext.eq_ignore_ascii_case(img_ext)) {
                                return true;
                            }
                        }
                        
                        // Check MIME type if available
                        if let Some(mime) = mime {
                            if mime.starts_with("image/") {
                                return true;
                            }
                        }
                        
                        false
                    })
                    .count();
                
                let total_size: i64 = attachments.iter()
                    .filter_map(|a| a.file.as_ref())
                    .filter_map(|f| f.size)
                    .sum();
                
                let size_text = if total_size > 1024 * 1024 {
                    format!("{:.1} MB", total_size as f64 / (1024.0 * 1024.0))
                } else if total_size > 1024 {
                    format!("{:.1} KB", total_size as f64 / 1024.0)
                } else {
                    format!("{} bytes", total_size)
                };
                
                details_lines.push(Line::from(vec![
                    Span::styled("Attachments: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("📎", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(" {} file(s)", attachments.len()))
                ]));
                
                if image_count > 0 {
                    details_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("🖼️", Style::default().fg(Color::Green)),
                        Span::raw(format!(" {} image(s)", image_count))
                    ]));
                }
                
                details_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("💾", Style::default().fg(Color::Cyan)),
                    Span::raw(format!(" Total size: {}", size_text))
                ]));
                
                // Show first few attachment names
                let max_show = 2;
                for (_i, attachment) in attachments.iter().take(max_show).enumerate() {
                    if let Some(file) = &attachment.file {
                        let file_name = file.name.as_deref().unwrap_or("Unknown file");
                        let icon = if {
                            let image_extensions = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico", "tiff", "tga"];
                            
                            // Check file extension
                            if let Some(ext) = file_name.split('.').last() {
                                if image_extensions.iter().any(|&img_ext| ext.eq_ignore_ascii_case(img_ext)) {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                // Check MIME type if available
                                if let Some(mime) = file.mime.as_deref() {
                                    mime.starts_with("image/")
                                } else {
                                    false
                                }
                            }
                        } {
                            "🖼️"
                        } else {
                            "📎"
                        };
                        
                        details_lines.push(Line::from(vec![
                            Span::raw("  "),
                            Span::styled(icon, Style::default().fg(Color::Yellow)),
                            Span::raw(" "),
                            Span::styled(file_name, Style::default().fg(Color::Blue))
                        ]));
                    }
                }
                
                if attachments.len() > max_show {
                    details_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled("...", Style::default().fg(Color::Gray)),
                        Span::raw(format!(" and {} more", attachments.len() - max_show))
                    ]));
                }
                
                details_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("Press '.a' to view all attachments", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Comments
        if let Some(comments) = &task.comments {
            if !comments.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Comments: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("💬", Style::default().fg(Color::Green)),
                    Span::raw(format!(" {} comment(s)", comments.len()))
                ]));
                details_lines.push(Line::from(""));

                for (_i, comment) in comments.iter().enumerate() {
                    // Author
                    let author = if let Some(user) = &comment.author {
                        if !user.username.is_empty() {
                            if let Some(name) = &user.name {
                                if !name.is_empty() {
                                    format!("{} (@{})", name, user.username)
                                } else {
                                    format!("@{}", user.username)
                                }
                            } else {
                                format!("@{}", user.username)
                            }
                        } else {
                            "Unknown user".to_string()
                        }
                    } else {
                        "Unknown user".to_string()
                    };
                    // Date
                    let date_str = if let Some(created) = &comment.created {
                        if !created.is_empty() {
                            if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(created) {
                                parsed_date.format("%Y-%m-%d %H:%M").to_string()
                            } else {
                                created.clone()
                            }
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };
                    // Text
                    let text = comment.comment.as_deref().unwrap_or("");

                    details_lines.push(Line::from(vec![
                        Span::raw("  ─ "),
                        Span::styled(author.clone(), Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(date_str.clone(), Style::default().fg(Color::DarkGray)),
                    ]));
                    details_lines.push(Line::from(vec![
                        Span::raw("     "),
                        Span::raw(text),
                    ]));
                    details_lines.push(Line::from(""));
                }
            }
        }

        // Reminders
        if let Some(reminders) = &task.reminders {
            if !reminders.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Reminders: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("🔔", Style::default().fg(Color::Magenta)),
                    Span::raw(format!(" {} reminder(s)", reminders.len()))
                ]));
                
                // Show reminder details if there are few reminders
                if reminders.len() <= 3 {
                    for reminder in reminders {
                        let relative_text = match reminder.relative_to.as_deref() {
                            Some("due_date") => "before due date",
                            Some("start_date") => "before start date",
                            Some("end_date") => "before end date",
                            _ => "relative",
                        };
                        
                        if let Some(relative_period) = reminder.relative_period {
                            let period_text = if relative_period < 60 {
                                format!("{} second(s)", relative_period)
                            } else if relative_period < 3600 {
                                format!("{} minute(s)", relative_period / 60)
                            } else if relative_period < 86400 {
                                format!("{} hour(s)", relative_period / 3600)
                            } else {
                                format!("{} day(s)", relative_period / 86400)
                            };
                            
                            details_lines.push(Line::from(vec![
                                Span::raw("  • "),
                                Span::raw(format!("{} {}", period_text, relative_text))
                            ]));
                        }
                    }
                }
                details_lines.push(Line::from(""));
            }
        }

        // Bucket information
        if let Some(bucket_id) = task.bucket_id {
            if bucket_id > 0 {
                details_lines.push(Line::from(vec![
                    Span::styled("Bucket ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(bucket_id.to_string())
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Repeat settings
        if let Some(repeat_after) = task.repeat_after {
            if repeat_after > 0 {
                let repeat_text = if let Some(repeat_mode) = task.repeat_mode {
                    match repeat_mode {
                        0 => {
                            if repeat_after < 60 {
                                format!("Every {} second(s)", repeat_after)
                            } else if repeat_after < 3600 {
                                format!("Every {} minute(s)", repeat_after / 60)
                            } else {
                                format!("Every {} hour(s)", repeat_after / 3600)
                            }
                        },
                        1 => format!("Every {} day(s)", repeat_after),
                        2 => format!("Every {} week(s)", repeat_after),
                        3 => format!("Every {} month(s)", repeat_after),
                        4 => format!("Every {} year(s)", repeat_after),
                        _ => format!("Custom repeat: {} seconds", repeat_after),
                    }
                } else {
                    format!("Repeats every {} seconds", repeat_after)
                };
                details_lines.push(Line::from(vec![
                    Span::styled("Repeat: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("🔁", Style::default().fg(Color::Cyan)),
                    Span::raw(format!(" {}", repeat_text))
                ]));
                details_lines.push(Line::from(""));
            }
        }

        // Created by and date
        if let Some(created_by) = &task.created_by {
            let creator_name = if let Some(name) = &created_by.name {
                if !name.is_empty() {
                    format!("{} (@{})", name, created_by.username)
                } else {
                    format!("@{}", created_by.username)
                }
            } else {
                format!("@{}", created_by.username)
            };
            details_lines.push(Line::from(vec![
                Span::styled("Created by: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(creator_name, Style::default().fg(Color::Cyan))
            ]));
            
            if let Some(created) = &task.created {
                if !created.is_empty() {
                    // Try to parse and format the date nicely
                    if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(created) {
                        details_lines.push(Line::from(vec![
                            Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
                        ]));
                    } else {
                        details_lines.push(Line::from(vec![
                            Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(created)
                        ]));
                    }
                }
            }
            details_lines.push(Line::from(""));
        } else if let Some(created) = &task.created {
            if !created.is_empty() {
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(created) {
                    details_lines.push(Line::from(vec![
                        Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
                    ]));
                } else {
                    details_lines.push(Line::from(vec![
                        Span::styled("Created: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(created)
                    ]));
                }
                details_lines.push(Line::from(""));
            }
        }

        if let Some(updated) = &task.updated {
            if !updated.is_empty() {
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(updated) {
                    details_lines.push(Line::from(vec![
                        Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
                    ]));
                } else {
                    details_lines.push(Line::from(vec![
                        Span::styled("Updated: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(updated)
                    ]));
                }
                details_lines.push(Line::from(""));
            }
        }

        // Task ID and Identifier
        if let Some(identifier) = &task.identifier {
            if !identifier.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} ({})", task.id, identifier))
                ]));
            } else {
                details_lines.push(Line::from(vec![
                    Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(task.id.to_string())
                ]));
            }
        } else {
            details_lines.push(Line::from(vec![
                Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(task.id.to_string())
            ]));
        }

        // Position/Index
        if let Some(index) = task.index {
            if index > 0 {
                details_lines.push(Line::from(vec![
                    Span::styled("Index: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(index.to_string())
                ]));
            }
        }

        details_lines
    } else {
        vec![Line::from("No task selected")]
    };
    let paragraph = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Task Details"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

