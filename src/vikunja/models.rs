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
    pub hex_color: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct User {
    #[allow(dead_code)]
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub project_id: i64,
    pub labels: Option<Vec<Label>>,
    pub assignees: Option<Vec<User>>,
    pub priority: Option<i32>,
    #[serde(deserialize_with = "deserialize_optional_datetime")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub is_favorite: bool,
}
