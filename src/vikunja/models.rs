use serde::Deserialize;

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
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub project_id: i64,
    pub labels: Option<Vec<Label>>,
}
