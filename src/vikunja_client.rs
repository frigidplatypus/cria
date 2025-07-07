use reqwest::{Client, Result as ReqwestResult};
use crate::vikunja_parser::{QuickAddParser};
use crate::debug::debug_log;

// --- Task-related types and functions ---
pub mod tasks;
pub use tasks::*;

// --- Project-related types and functions ---
pub mod projects;

// --- Filter-related types and functions ---
pub mod filters;

// --- User-related types and functions ---
pub mod users;
pub use users::*;

// --- Label-related types and functions ---
pub mod labels;

pub struct VikunjaClient {
    client: Client,
    base_url: String,
    auth_token: String,
    parser: QuickAddParser,
}

#[allow(dead_code)]
/// Create a quick task via the Vikunja API
pub async fn create_quick_task(
    base_url: String,
    auth_token: String,
    task_text: String,
    project_id: i64,
) -> Result<(), anyhow::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/tasks", base_url);
    let payload = serde_json::json!({
        "title": task_text,
        "project_id": project_id,
    });
    let resp = client
        .post(&url)
        .bearer_auth(auth_token)
        .json(&payload)
        .send()
        .await?;
    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to create task: {}", resp.status()))
    }
}

impl VikunjaClient {
    pub fn new(base_url: String, auth_token: String) -> Self {
        debug_log(&format!("Creating VikunjaClient with URL: {}", base_url));
        debug_log(&format!("Auth token length: {}", auth_token.len()));
        Self {
            client: Client::new(),
            base_url,
            auth_token,
            parser: QuickAddParser::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn auth_token(&self) -> &str {
        &self.auth_token
    }

    pub async fn test_connection(&self) -> ReqwestResult<bool> {
        debug_log(&format!("Testing connection to {}", self.base_url));
        let url = format!("{}/api/v1/info", self.base_url);
        debug_log(&format!("Testing with URL: {}", url));
        let response = self.client
            .get(&url)
            .send()
            .await;
        match response {
            Ok(resp) => {
                debug_log(&format!("Connection test - Status: {}", resp.status()));
                if resp.status().is_success() {
                    debug_log("Connection successful!");
                    Ok(true)
                } else {
                    debug_log(&format!("Connection failed with status: {}", resp.status()));
                    Ok(false)
                }
            },
            Err(e) => {
                debug_log(&format!("Connection test failed: {:?}", e));
                if e.is_connect() {
                    debug_log(&format!("Cannot connect to Vikunja at {}. Is it running?", self.base_url));
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    #[tokio::test]
    async fn test_magic_task_creation() {
        // This would need a real Vikunja instance to test against
        // let client = VikunjaClient::new(
        //     "https://vikunja.example.com".to_string(),
        //     "your-auth-token".to_string()
        // );
        // let task = client.create_task_with_magic(
        //     "Buy groceries *shopping @john +personal tomorrow !2",
        //     1
        // ).await.unwrap();
        // assert_eq!(task.title, "Buy groceries");
        // assert_eq!(task.priority, Some(2));
    }
}
