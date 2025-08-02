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

// --- Attachment-related types and functions ---
pub mod attachments;
pub use attachments::*;

// --- Relation-related types and functions ---
// DISABLED: Incomplete feature
// pub mod relations;
// pub use relations::*;

pub struct VikunjaClient {
    client: Client,
    base_url: String,
    auth_token: String,
    parser: QuickAddParser,
    attachment_client: AttachmentClient,
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
        let client = Client::new();
        let attachment_client = AttachmentClient::new(
            client.clone(),
            base_url.clone(),
            auth_token.clone(),
        );
        Self {
            client,
            base_url,
            auth_token,
            parser: QuickAddParser::new(),
            attachment_client,
        }
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[allow(dead_code)]
    pub fn client(&self) -> &Client {
        &self.client
    }

    #[allow(dead_code)]
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

    // Attachment methods
    pub async fn get_task_attachments(&self, task_id: i64) -> Result<Vec<crate::vikunja::models::Attachment>, Box<dyn std::error::Error + Send + Sync>> {
        self.attachment_client.get_task_attachments(task_id).await
    }

    pub async fn upload_attachment(&self, task_id: i64, file_path: &std::path::Path) -> Result<crate::vikunja::models::Attachment, Box<dyn std::error::Error + Send + Sync>> {
        self.attachment_client.upload_attachment(task_id, file_path).await
    }

    pub async fn download_attachment(&self, attachment: &crate::vikunja::models::Attachment, download_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.attachment_client.download_attachment(attachment, download_path).await
    }

    /// Remove an attachment from a task
    pub async fn remove_attachment(&self, task_id: i64, attachment_id: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.attachment_client.remove_attachment(task_id, attachment_id).await
    }

    /// Get attachment metadata for a task
    pub async fn get_attachment(&self, task_id: i64, attachment_id: i64) -> Result<crate::vikunja::models::Attachment, Box<dyn std::error::Error + Send + Sync>> {
        self.attachment_client.get_attachment(task_id, attachment_id).await
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
