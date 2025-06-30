// Project-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use crate::debug::debug_log;
use serde::{Deserialize, Serialize};

// --- Project-related types and functions ---
// VikunjaProject, project-related impls and functions
// ...move all VikunjaProject and project API impls here...

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaProject {
    pub id: i64, // Changed from u64 to i64 to support negative IDs
    pub title: String,
}

impl super::VikunjaClient {
    pub async fn find_or_get_project_id(&self, project_name: &str) -> reqwest::Result<Option<i64>> {
        let url = format!("{}/api/v1/projects", self.base_url);
        let normalized_input = project_name.trim().to_ascii_lowercase();
        debug_log(&format!("Looking for project: '{}' (normalized: '{}')", project_name, normalized_input));
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let text = response.text().await?;
        debug_log(&format!("Raw project list response: {}", text));
        let projects: Vec<VikunjaProject> = match serde_json::from_str(&text) {
            Ok(projects) => projects,
            Err(e) => {
                debug_log(&format!("Failed to decode project list: {}", e));
                return Ok(None);
            }
        };
        debug_log(&format!("Available projects: {:?}", projects.iter().map(|p| format!("{} (id={})", p.title, p.id)).collect::<Vec<_>>()));
        Ok(projects.iter()
            .filter(|p| p.id > 0)
            .find(|p| p.title.trim().to_ascii_lowercase() == normalized_input)
            .map(|p| p.id))
    }

    #[allow(dead_code)]
    pub async fn get_all_projects(&self) -> reqwest::Result<Vec<VikunjaProject>> {
        let url = format!("{}/api/v1/projects", self.base_url);
        let resp = self.client.get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;
        resp.json::<Vec<VikunjaProject>>().await
    }

    #[allow(dead_code)]
    pub async fn create_project(&self, title: &str, color: &str) -> reqwest::Result<VikunjaProject> {
        let url = format!("{}/api/v1/projects", self.base_url);
        let payload = serde_json::json!({"title": title, "color": color});
        let resp = self.client.post(&url)
            .bearer_auth(&self.auth_token)
            .json(&payload)
            .send()
            .await?;
        resp.json::<VikunjaProject>().await
    }
}

// --- Project-related API impls ---
// All project-related methods from VikunjaClient impl go here.
// ...existing code...
