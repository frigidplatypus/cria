// Filter-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FilterProject {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
}

impl super::VikunjaClient {
    pub async fn get_saved_filters(&self) -> reqwest::Result<Vec<(i64, String, Option<String>)>> {
        let url = format!("{}/api/v1/projects", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let text = response.text().await?;
        crate::debug::debug_log(&format!("Raw /api/v1/projects response: {} characters", text.len()));
        let projects: Result<Vec<FilterProject>, _> = serde_json::from_str(&text);
        match projects {
            Ok(projects) => {
                let filters: Vec<_> = projects
                    .into_iter()
                    .filter(|p| p.id < 0)
                    .map(|f| (f.id, f.title, f.description))
                    .collect();
                crate::debug::debug_log(&format!("Extracted {} filters from projects", filters.len()));
                Ok(filters)
            },
            Err(e) => {
                crate::debug::debug_log(&format!("Failed to deserialize projects: {}", e));
                Ok(vec![])
            }
        }
    }
    #[allow(dead_code)]
    pub async fn get_tasks_for_filter(&self, filter_id: i64) -> reqwest::Result<Vec<crate::vikunja::models::Task>> {
        let url = if filter_id < 0 {
            format!("{}/api/v1/projects/{}/tasks", self.base_url, filter_id)
        } else {
            format!("{}/api/v1/filter/{}/tasks", self.base_url, filter_id)
        };
        crate::debug::debug_log(&format!("get_tasks_for_filter: Fetching tasks from {}", url));
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let status = response.status();
        let text = {
            response.error_for_status_ref()?;
            response.text().await?
        };
        crate::debug::debug_log(&format!("get_tasks_for_filter: Response status: {}", status));
        crate::debug::debug_log(&format!("get_tasks_for_filter: Response body: {} characters", text.len()));
        match serde_json::from_str::<Vec<crate::vikunja::models::Task>>(&text) {
            Ok(tasks) => Ok(tasks),
            Err(e) => {
                crate::debug::debug_log(&format!("get_tasks_for_filter: JSON decode error: {}", e));
                Ok(vec![])
            }
        }
    }
}

// --- Filter-related API impls ---
// All filter-related methods from VikunjaClient impl go here.
// ...existing code...
