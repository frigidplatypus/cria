use crate::vikunja::models::{Task, Project};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use std::env;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VikunjaClient {
    client: Client,
    api_url: String,
}

impl VikunjaClient {
    pub fn new() -> Self {
        let api_url = env::var("VIKUNJA_API_URL").expect("VIKUNJA_API_URL must be set");
        let api_token = env::var("VIKUNJA_API_TOKEN").expect("VIKUNJA_API_TOKEN must be set");

        let mut headers = HeaderMap::new();
        let auth_header = format!("Bearer {}", api_token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_header).expect("Failed to create auth header"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build client");

        Self { client, api_url }
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>, reqwest::Error> {
        let url = format!("{}/projects", self.api_url);
        let projects = self.client.get(&url).send().await?.json::<Vec<Project>>().await?;
        Ok(projects)
    }

    pub async fn get_tasks(&self) -> Result<Vec<Task>, reqwest::Error> {
        let url = format!("{}/tasks/all", self.api_url);
        let tasks = self.client.get(&url).send().await?.json::<Vec<Task>>().await?;
        Ok(tasks)
    }

    pub async fn get_tasks_with_projects(&self) -> Result<(Vec<Task>, HashMap<i64, String>, HashMap<i64, String>), reqwest::Error> {
        let projects = self.get_projects().await?;
        let tasks = self.get_tasks().await?;
        
        let project_map: HashMap<i64, String> = projects
            .iter()
            .map(|p| (p.id, p.title.clone()))
            .collect();
        
        let project_colors: HashMap<i64, String> = projects
            .into_iter()
            .map(|p| (p.id, p.hex_color))
            .collect();
        
        Ok((tasks, project_map, project_colors))
    }
}
