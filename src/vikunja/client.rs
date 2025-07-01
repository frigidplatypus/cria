use crate::vikunja::models::{Task, Project};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Clone)]
#[allow(dead_code)] // Alternative client implementation
pub struct VikunjaClient {
    client: Client,
    api_url: String,
}

impl VikunjaClient {
    #[allow(dead_code)] // Alternative client methods
    pub async fn get_projects(&self) -> Result<Vec<Project>, reqwest::Error> {
        let url = format!("{}/projects", self.api_url);
        let projects = self.client.get(&url).send().await?.json::<Vec<Project>>().await?;
        Ok(projects)
    }

    #[allow(dead_code)] // Alternative client methods
    pub async fn get_tasks(&self) -> Result<Vec<Task>, reqwest::Error> {
        let url = format!("{}/tasks/all", self.api_url);
        let tasks = self.client.get(&url).send().await?.json::<Vec<Task>>().await?;
        Ok(tasks)
    }

    #[allow(dead_code)] // Alternative client methods
    pub async fn get_tasks_filtered(&self, show_completed: bool) -> Result<Vec<Task>, reqwest::Error> {
        let filter_param = if show_completed { "true" } else { "false" };
        let url = format!("{}/tasks/all?filter_done={}", self.api_url, filter_param);
        let tasks = self.client.get(&url).send().await?.json::<Vec<Task>>().await?;
        Ok(tasks)
    }

    #[allow(dead_code)] // Alternative client methods
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

    #[allow(dead_code)] // Alternative client methods
    pub async fn get_tasks_with_projects_filtered(&self, show_completed: bool) -> Result<(Vec<Task>, HashMap<i64, String>, HashMap<i64, String>), reqwest::Error> {
        let projects = self.get_projects().await?;
        let tasks = self.get_tasks_filtered(show_completed).await?;
        
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
