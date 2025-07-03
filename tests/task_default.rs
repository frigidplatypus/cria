use cria::vikunja::models::Task;
use chrono::{DateTime, Utc};

impl Default for Task {
    fn default() -> Self {
        Task {
            id: 0,
            title: String::new(),
            description: None,
            done: false,
            done_at: None,
            project_id: 0,
            labels: None,
            assignees: None,
            priority: None,
            due_date: None,
            start_date: None,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: false,
            position: None,
            index: None,
            identifier: None,
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: None,
            buckets: None,
            attachments: None,
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: None,
            subscription: None,
        }
    }
}
