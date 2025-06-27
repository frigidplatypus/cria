// Filter-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FilterProject {
    pub id: i64,
    pub title: String,
}

impl super::VikunjaClient {
    pub async fn get_saved_filters(&self) -> reqwest::Result<Vec<(i64, String)>> {
        let url = format!("{}/api/v1/projects", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let projects: Vec<FilterProject> = response.json().await?;
        // Filters are projects with negative IDs
        Ok(projects
            .into_iter()
            .filter(|p| p.id < 0)
            .map(|f| (f.id, f.title))
            .collect())
    }
    pub async fn get_tasks_for_filter(&self, filter_id: i64) -> reqwest::Result<Vec<crate::vikunja::models::Task>> {
        let url = format!("{}/api/v1/filter/{}/tasks", self.base_url, filter_id);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let tasks: Vec<crate::vikunja::models::Task> = response.json().await?;
        Ok(tasks)
    }
}

// --- Filter-related API impls ---
// All filter-related methods from VikunjaClient impl go here.
// ...existing code...
