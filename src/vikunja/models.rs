use serde::{Deserialize, Deserializer};
use chrono::{DateTime, Utc, Datelike};

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.is_empty() => Ok(None),
        Some(s) => {
            match DateTime::parse_from_rfc3339(&s) {
                Ok(dt) => {
                    // Check if it's the epoch start or year 1 (typical for null dates)
                    if dt.year() <= 1900 {
                        Ok(None)
                    } else {
                        Ok(Some(dt.with_timezone(&Utc)))
                    }
                }
                Err(_) => Ok(None),
            }
        }
        None => Ok(None),
    }
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Project {
    pub id: i64,
    pub title: String,
    pub hex_color: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Label {
    pub id: i64,
    pub title: String,
    pub hex_color: Option<String>,
    pub description: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub created_by: Option<User>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct User {
    pub id: i64,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Task {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub done: bool,
    pub done_at: Option<String>,
    pub project_id: i64,
    pub labels: Option<Vec<Label>>,
    pub assignees: Option<Vec<User>>,
    pub priority: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub created_by: Option<User>,
    pub percent_done: Option<u8>,
    pub is_favorite: bool,
    pub position: Option<i64>,
    pub index: Option<i64>,
    pub identifier: Option<String>,
    pub hex_color: Option<String>,
    pub cover_image_attachment_id: Option<i64>,
    pub bucket_id: Option<i64>,
    pub buckets: Option<Vec<Bucket>>,
    pub attachments: Option<Vec<Attachment>>,
    pub comments: Option<Vec<Comment>>,
    pub reactions: Option<std::collections::HashMap<String, Vec<User>>>,
    pub related_tasks: Option<std::collections::HashMap<String, Vec<Task>>>,
    pub reminders: Option<Vec<Reminder>>,
    pub repeat_after: Option<i64>,
    pub repeat_mode: Option<i64>,
    pub subscription: Option<Subscription>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Attachment {
    pub id: i64,
    pub task_id: i64,
    pub created: Option<String>,
    pub created_by: Option<User>,
    pub file: Option<FileAttachment>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct FileAttachment {
    pub id: i64,
    pub name: Option<String>,
    pub mime: Option<String>,
    pub size: Option<i64>,
    pub created: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Comment {
    pub id: i64,
    pub author: Option<User>,
    pub comment: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub reactions: Option<std::collections::HashMap<String, Vec<User>>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Bucket {
    pub id: i64,
    pub title: Option<String>,
    pub position: Option<i64>,
    pub limit: Option<i64>,
    pub count: Option<i64>,
    pub project_view_id: Option<i64>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub created_by: Option<User>,
    pub tasks: Option<Vec<Task>>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Reminder {
    pub reminder: Option<String>,
    pub relative_to: Option<String>,
    pub relative_period: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)] // API response fields may not all be used
pub struct Subscription {
    pub id: i64,
    pub entity: Option<i64>,
    pub entity_id: Option<i64>,
    pub created: Option<String>,
}

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

impl Task {
    pub fn to_vikunja_task(&self) -> crate::vikunja_client::tasks::VikunjaTask {
        crate::vikunja_client::tasks::VikunjaTask {
            id: Some(self.id as u64),
            title: self.title.clone(),
            description: self.description.clone(),
            done: Some(self.done),
            priority: self.priority.map(|p| p as u8),
            due_date: self.due_date,
            start_date: self.start_date,
            project_id: self.project_id as u64,
            labels: self.labels.as_ref().map(|labels| labels.iter().map(|l| crate::vikunja_client::tasks::VikunjaLabel {
                id: Some(l.id as u64),
                title: l.title.clone(),
                hex_color: l.hex_color.clone(),
            }).collect()),
            assignees: self.assignees.as_ref().map(|assignees| assignees.iter().map(|a| crate::vikunja_client::VikunjaUser {
                id: Some(a.id as u64),
                username: a.username.clone(),
                name: a.name.clone(),
                email: a.email.clone(),
            }).collect()),
            is_favorite: Some(self.is_favorite),
        }
    }
    pub fn from_vikunja_task(vikunja_task: crate::vikunja_client::tasks::VikunjaTask) -> Self {
        Self {
            id: vikunja_task.id.unwrap_or(0) as i64,
            title: vikunja_task.title,
            description: vikunja_task.description,
            done: vikunja_task.done.unwrap_or(false),
            done_at: None,
            project_id: vikunja_task.project_id as i64,
            labels: vikunja_task.labels.map(|labels| labels.into_iter().map(|l| Label {
                id: l.id.unwrap_or(0) as i64,
                title: l.title,
                hex_color: l.hex_color,
                description: None,
                created: None,
                updated: None,
                created_by: None,
            }).collect()),
            assignees: vikunja_task.assignees.map(|assignees| assignees.into_iter().map(|a| User {
                id: a.id.unwrap_or(0) as i64,
                username: a.username,
                name: a.name,
                email: a.email,
                created: None,
                updated: None,
            }).collect()),
            priority: vikunja_task.priority.map(|p| p as i32),
            due_date: vikunja_task.due_date,
            start_date: vikunja_task.start_date,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: vikunja_task.is_favorite.unwrap_or(false),
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
