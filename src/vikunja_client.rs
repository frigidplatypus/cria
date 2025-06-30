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

// Helper function for easy usage
pub async fn create_quick_task(
    vikunja_url: &str,
    auth_token: &str,
    magic_text: &str,
    default_project_id: u64,
) -> ReqwestResult<crate::vikunja_client::tasks::VikunjaTask> {
    let client = VikunjaClient::new(vikunja_url.to_string(), auth_token.to_string());
    client.create_task_with_magic(magic_text, default_project_id).await
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
