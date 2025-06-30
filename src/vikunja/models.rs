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
pub struct Project {
    pub id: i64,
    pub title: String,
    pub hex_color: String,
}

#[derive(Deserialize, Debug, Clone)]
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
pub struct User {
    pub id: i64,
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
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
    pub start_date: Option<String>,
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
pub struct Attachment {
    pub id: i64,
    pub task_id: i64,
    pub created: Option<String>,
    pub created_by: Option<User>,
    pub file: Option<FileAttachment>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileAttachment {
    pub id: i64,
    pub name: Option<String>,
    pub mime: Option<String>,
    pub size: Option<i64>,
    pub created: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Comment {
    pub id: i64,
    pub author: Option<User>,
    pub comment: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub reactions: Option<std::collections::HashMap<String, Vec<User>>>,
}

#[derive(Deserialize, Debug, Clone)]
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
pub struct Reminder {
    pub reminder: Option<String>,
    pub relative_to: Option<String>,
    pub relative_period: Option<i64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Subscription {
    pub id: i64,
    pub entity: Option<i64>,
    pub entity_id: Option<i64>,
    pub created: Option<String>,
}
