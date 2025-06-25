use reqwest::{Client, Result as ReqwestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::vikunja_parser::{QuickAddParser};
use crate::debug::debug_log;

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaTask {
    pub id: Option<u64>,
    pub title: String,
    pub description: Option<String>,
    pub done: Option<bool>,
    pub priority: Option<u8>,
    pub due_date: Option<DateTime<Utc>>,
    pub project_id: u64,
    pub labels: Option<Vec<VikunjaLabel>>,
    pub assignees: Option<Vec<VikunjaUser>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaLabel {
    pub id: Option<u64>,
    pub title: String,
    pub hex_color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaUser {
    pub id: Option<u64>,
    pub username: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaProject {
    pub id: u64,
    pub title: String,
}

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

    /// Create a task using Quick Add Magic syntax
    pub async fn create_task_with_magic(
        &self,
        magic_text: &str,
        default_project_id: u64,
    ) -> ReqwestResult<VikunjaTask> {
        debug_log(&format!("Parsing magic text: '{}'", magic_text));
        let parsed = self.parser.parse(magic_text);
        debug_log(&format!("Parsed task - title: '{}', labels: {:?}, project: {:?}, priority: {:?}", 
                 parsed.title, parsed.labels, parsed.project, parsed.priority));
        
        // Step 1: Determine project ID
        let project_id = if let Some(project_name) = &parsed.project {
            debug_log(&format!("Looking up project: '{}'", project_name));
            match self.find_or_get_project_id(project_name).await {
                Ok(Some(id)) => {
                    debug_log(&format!("Found project ID: {}", id));
                    id
                }
                Ok(None) => {
                    debug_log(&format!("Project '{}' not found, using default: {}", project_name, default_project_id));
                    default_project_id
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project: {}, using default: {}", e, default_project_id));
                    default_project_id
                }
            }
        } else {
            debug_log(&format!("No project specified, using default: {}", default_project_id));
            default_project_id
        };

        // Step 2: Create the basic task
        let task = VikunjaTask {
            id: None,
            title: parsed.title.clone(),
            description: None,
            done: Some(false),
            priority: parsed.priority,
            due_date: parsed.due_date,
            project_id,
            labels: None,
            assignees: None,
        };

        debug_log(&format!("Creating task with project_id: {}, title: '{}'", project_id, task.title));
        let created_task = self.create_task(&task).await?;
        debug_log(&format!("Task created with ID: {:?}", created_task.id));
        
        let task_id = created_task.id.unwrap();

        // Step 3: Add labels
        for label_name in &parsed.labels {
            if let Ok(label) = self.ensure_label_exists(label_name).await {
                let _ = self.add_label_to_task(task_id, label.id.unwrap()).await;
            }
        }

        // Step 4: Add assignees
        for username in &parsed.assignees {
            if let Some(user) = self.find_user_by_username(username).await {
                let _ = self.add_assignee_to_task(task_id, user.id.unwrap()).await;
            }
        }

        // Step 5: Handle repeating tasks (if needed)
        if let Some(_repeat) = &parsed.repeat_interval {
            // Implement repeat logic based on Vikunja's repeat API
            // This would involve setting repeat_after or repeat_mode fields
        }

        // Return the updated task
        self.get_task(task_id).await
    }

    async fn create_task(&self, task: &VikunjaTask) -> ReqwestResult<VikunjaTask> {
        let url = format!("{}/api/v1/projects/{}/tasks", self.base_url, task.project_id);
        debug_log(&format!("Making PUT request to: {}", url));
        debug_log(&format!("Task payload: {:?}", task));
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(task)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                debug_log(&format!("Response status: {}", status));
                debug_log(&format!("Response headers: {:?}", resp.headers()));
                
                if resp.status().is_success() {
                    let result = resp.json::<VikunjaTask>().await;
                    match &result {
                        Ok(created_task) => {
                            debug_log(&format!("Successfully created task: {:?}", created_task));
                        }
                        Err(e) => {
                            debug_log(&format!("Failed to parse response JSON: {}", e));
                        }
                    }
                    result
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
                    debug_log(&format!("API error response ({}): {}", status, error_text));
                    // Return a connection error since we can't easily create custom reqwest errors
                    let fake_response = self.client.get("http://invalid-url-that-will-fail").send().await;
                    Err(fake_response.unwrap_err())
                }
            },
            Err(e) => {
                debug_log(&format!("Request failed with error: {:?}", e));
                debug_log(&format!("Error source: {:?}", e.source()));
                if e.is_connect() {
                    debug_log(&format!("This is a connection error - is Vikunja running on {}?", self.base_url));
                }
                if e.is_timeout() {
                    debug_log(&format!("This is a timeout error"));
                }
                if e.is_request() {
                    debug_log(&format!("This is a request building error"));
                }
                Err(e)
            }
        }
    }

    async fn get_task(&self, task_id: u64) -> ReqwestResult<VikunjaTask> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        response.json().await
    }

    async fn find_or_get_project_id(&self, project_name: &str) -> ReqwestResult<Option<u64>> {
        let url = format!("{}/api/v1/projects", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let projects: Vec<VikunjaProject> = response.json().await?;
        
        Ok(projects.iter()
            .find(|p| p.title.eq_ignore_ascii_case(project_name))
            .map(|p| p.id))
    }

    async fn ensure_label_exists(&self, label_name: &str) -> ReqwestResult<VikunjaLabel> {
        // First, try to find existing label
        if let Ok(Some(label)) = self.find_label_by_name(label_name).await {
            return Ok(label);
        }

        // Create new label if it doesn't exist
        self.create_label(label_name).await
    }

    async fn find_label_by_name(&self, label_name: &str) -> ReqwestResult<Option<VikunjaLabel>> {
        let url = format!("{}/api/v1/labels", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        let labels: Vec<VikunjaLabel> = response.json().await?;
        
        Ok(labels.into_iter()
            .find(|l| l.title.eq_ignore_ascii_case(label_name)))
    }

    async fn create_label(&self, label_name: &str) -> ReqwestResult<VikunjaLabel> {
        let url = format!("{}/api/v1/labels", self.base_url);
        
        let label = VikunjaLabel {
            id: None,
            title: label_name.to_string(),
            hex_color: None, // Could generate random color
        };

        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&label)
            .send()
            .await?;

        response.json().await
    }

    async fn add_label_to_task(&self, task_id: u64, label_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/labels", self.base_url, task_id);
        
        let label_task = HashMap::from([
            ("label_id", label_id),
        ]);

        let _response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&label_task)
            .send()
            .await?;

        Ok(())
    }

    async fn find_user_by_username(&self, username: &str) -> Option<VikunjaUser> {
        let url = format!("{}/api/v1/users/search/{}", self.base_url, username);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await.ok()?;

        let users: Vec<VikunjaUser> = response.json().await.ok()?;
        
        users.into_iter()
            .find(|u| u.username.eq_ignore_ascii_case(username))
    }

    async fn add_assignee_to_task(&self, task_id: u64, user_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/assignees", self.base_url, task_id);
        
        let assignee = HashMap::from([
            ("user_id", user_id),
        ]);

        let _response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&assignee)
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_task_with_magic(
        &self,
        task_id: i64,
        magic_text: &str,
    ) -> ReqwestResult<VikunjaTask> {
        debug_log(&format!("Updating task {} with magic text: '{}'", task_id, magic_text));
        let parsed = self.parser.parse(magic_text);
        debug_log(&format!("Parsed task - title: '{}', labels: {:?}, project: {:?}, priority: {:?}", 
                 parsed.title, parsed.labels, parsed.project, parsed.priority));
        
        // Step 1: Get the current task to preserve fields we're not updating
        let current_task = self.get_task(task_id as u64).await?;
        debug_log(&format!("Retrieved current task: {:?}", current_task));
        
        // Step 2: Determine project ID
        let project_id = if let Some(project_name) = &parsed.project {
            debug_log(&format!("Looking up project: '{}'", project_name));
            match self.find_or_get_project_id(project_name).await {
                Ok(Some(id)) => {
                    debug_log(&format!("Found project ID: {}", id));
                    id
                }
                Ok(None) => {
                    debug_log(&format!("Project '{}' not found, keeping current: {}", project_name, current_task.project_id));
                    current_task.project_id
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project: {}, keeping current: {}", e, current_task.project_id));
                    current_task.project_id
                }
            }
        } else {
            debug_log(&format!("No project specified, keeping current: {}", current_task.project_id));
            current_task.project_id
        };

        // Step 3: Update the basic task fields
        let updated_task = VikunjaTask {
            id: Some(task_id as u64),
            title: parsed.title.clone(),
            description: current_task.description, // Preserve description
            done: current_task.done, // Preserve done status
            priority: parsed.priority.or(current_task.priority), // Use new priority if provided, otherwise keep current
            due_date: parsed.due_date.or(current_task.due_date), // Use new due date if provided, otherwise keep current
            project_id,
            labels: None, // Will be handled separately
            assignees: None, // Will be handled separately
        };

        debug_log(&format!("Updating task with project_id: {}, title: '{}'", project_id, updated_task.title));
        let updated_task = self.update_task(&updated_task).await?;
        debug_log(&format!("Task updated with ID: {:?}", updated_task.id));

        // Step 4: Clear existing labels and add new ones
        if !parsed.labels.is_empty() {
            // Remove all existing labels
            if let Some(existing_labels) = &current_task.labels {
                for label in existing_labels {
                    if let Some(label_id) = label.id {
                        let _ = self.remove_label_from_task(task_id as u64, label_id).await;
                    }
                }
            }
            
            // Add new labels
            for label_name in &parsed.labels {
                if let Ok(label) = self.ensure_label_exists(label_name).await {
                    let _ = self.add_label_to_task(task_id as u64, label.id.unwrap()).await;
                }
            }
        }

        // Step 5: Clear existing assignees and add new ones
        if !parsed.assignees.is_empty() {
            // Remove all existing assignees
            if let Some(existing_assignees) = &current_task.assignees {
                for assignee in existing_assignees {
                    if let Some(user_id) = assignee.id {
                        let _ = self.remove_assignee_from_task(task_id as u64, user_id).await;
                    }
                }
            }
            
            // Add new assignees
            for username in &parsed.assignees {
                if let Some(user) = self.find_user_by_username(username).await {
                    let _ = self.add_assignee_to_task(task_id as u64, user.id.unwrap()).await;
                }
            }
        }

        // Step 6: Handle repeating tasks (if needed)
        if let Some(_repeat) = &parsed.repeat_interval {
            // Implement repeat logic based on Vikunja's repeat API
            // This would involve setting repeat_after or repeat_mode fields
        }

        // Return the updated task
        self.get_task(task_id as u64).await
    }

    async fn update_task(&self, task: &VikunjaTask) -> ReqwestResult<VikunjaTask> {
        let task_id = task.id.unwrap();
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        debug_log(&format!("Making POST request to: {}", url));
        debug_log(&format!("Task payload: {:?}", task));
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(task)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                debug_log(&format!("Response status: {}", status));
                debug_log(&format!("Response headers: {:?}", resp.headers()));
                
                if resp.status().is_success() {
                    let result = resp.json::<VikunjaTask>().await;
                    match &result {
                        Ok(updated_task) => {
                            debug_log(&format!("Successfully updated task: {:?}", updated_task));
                        }
                        Err(e) => {
                            debug_log(&format!("Failed to parse response JSON: {}", e));
                        }
                    }
                    result
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
                    debug_log(&format!("API error response ({}): {}", status, error_text));
                    // Return a connection error since we can't easily create custom reqwest errors
                    let fake_response = self.client.get("http://invalid-url-that-will-fail").send().await;
                    Err(fake_response.unwrap_err())
                }
            },
            Err(e) => {
                debug_log(&format!("Request failed with error: {:?}", e));
                debug_log(&format!("Error source: {:?}", e.source()));
                if e.is_connect() {
                    debug_log(&format!("This is a connection error - is Vikunja running on {}?", self.base_url));
                }
                if e.is_timeout() {
                    debug_log(&format!("This is a timeout error"));
                }
                if e.is_request() {
                    debug_log(&format!("This is a request building error"));
                }
                Err(e)
            }
        }
    }

    async fn remove_label_from_task(&self, task_id: u64, label_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/labels/{}", self.base_url, task_id, label_id);
        
        let _response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        Ok(())
    }

    async fn remove_assignee_from_task(&self, task_id: u64, user_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/assignees/{}", self.base_url, task_id, user_id);
        
        let _response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        Ok(())
    }

    // Update task completion status
    pub async fn update_task_completion(&self, task_id: i64, done: bool) -> ReqwestResult<VikunjaTask> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        debug_log(&format!("Updating task {} completion to: {}", task_id, done));
        
        // Create minimal task update payload
        let task_update = serde_json::json!({
            "done": done
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&task_update)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                debug_log(&format!("Response status: {}", status));
                
                if resp.status().is_success() {
                    let result = resp.json::<VikunjaTask>().await;
                    match &result {
                        Ok(updated_task) => {
                            debug_log(&format!("Successfully updated task completion: {:?}", updated_task));
                        }
                        Err(e) => {
                            debug_log(&format!("Failed to parse response JSON: {}", e));
                        }
                    }
                    result
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
                    debug_log(&format!("API error response ({}): {}", status, error_text));
                    // Return a connection error since we can't easily create custom reqwest errors
                    let fake_response = self.client.get("http://invalid-url-that-will-fail").send().await;
                    Err(fake_response.unwrap_err())
                }
            },
            Err(e) => {
                debug_log(&format!("Request failed with error: {:?}", e));
                Err(e)
            }
        }
    }

    // Delete a task
    pub async fn delete_task(&self, task_id: i64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        debug_log(&format!("Deleting task with ID: {}", task_id));
        
        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await;

        match response {
            Ok(resp) => {
                let status = resp.status();
                debug_log(&format!("Response status: {}", status));
                
                if resp.status().is_success() {
                    debug_log(&format!("Successfully deleted task: {}", task_id));
                    Ok(())
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
                    debug_log(&format!("API error response ({}): {}", status, error_text));
                    // Return a connection error since we can't easily create custom reqwest errors
                    let fake_response = self.client.get("http://invalid-url-that-will-fail").send().await;
                    Err(fake_response.unwrap_err())
                }
            },
            Err(e) => {
                debug_log(&format!("Request failed with error: {:?}", e));
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
) -> ReqwestResult<VikunjaTask> {
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
        // 
        // let task = client.create_task_with_magic(
        //     "Buy groceries *shopping @john +personal tomorrow !2",
        //     1
        // ).await.unwrap();
        // 
        // assert_eq!(task.title, "Buy groceries");
        // assert_eq!(task.priority, Some(2));
    }
}
