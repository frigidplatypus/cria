// Task-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use reqwest::{Client, Result as ReqwestResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::vikunja_parser::QuickAddParser;
use crate::debug::debug_log;
use crate::vikunja_client::VikunjaUser;
use crate::vikunja_client::projects::VikunjaProject;

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

impl super::VikunjaClient {
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
        if let Some(project_name) = &parsed.project {
            debug_log(&format!("Magic syntax project: '{}'. Attempting lookup...", project_name));
        } else {
            debug_log("No project specified in magic syntax.");
        }
        let project_id = if let Some(project_name) = &parsed.project {
            debug_log(&format!("Looking up project: '{}'.", project_name));
            match self.find_or_get_project_id(project_name).await {
                Ok(Some(id)) => {
                    debug_log(&format!("Found project ID: {} for project '{}'.", id, project_name));
                    id
                }
                Ok(None) => {
                    debug_log(&format!("Project '{}' not found, using default: {}.", project_name, default_project_id));
                    default_project_id.try_into().unwrap()
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project '{}': {}. Using default: {}.", project_name, e, default_project_id));
                    default_project_id.try_into().unwrap()
                }
            }
        } else {
            debug_log(&format!("No project specified, using default: {}.", default_project_id));
            default_project_id.try_into().unwrap()
        };

        debug_log(&format!("Final project_id to use: {}", project_id));

        // Step 2: Create the basic task
        let task = VikunjaTask {
            id: None,
            title: parsed.title.clone(),
            description: None,
            done: Some(false),
            priority: parsed.priority,
            due_date: parsed.due_date,
            project_id: project_id.try_into().unwrap(),
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

    pub async fn create_task(&self, task: &VikunjaTask) -> ReqwestResult<VikunjaTask> {
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

    pub async fn get_task(&self, task_id: u64) -> ReqwestResult<VikunjaTask> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        response.json().await
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
        let current_task = self.get_task(task_id as u64).await?;
        debug_log(&format!("Retrieved current task: {:?}", current_task));
        let project_id = if let Some(project_name) = &parsed.project {
            debug_log(&format!("Looking up project: '{}', current: {}.", project_name, current_task.project_id));
            match self.find_or_get_project_id(project_name).await {
                Ok(Some(id)) => {
                    debug_log(&format!("Found project ID: {}", id));
                    id
                }
                Ok(None) => {
                    debug_log(&format!("Project '{}' not found, keeping current: {}", project_name, current_task.project_id));
                    current_task.project_id.try_into().unwrap()
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project: {}, keeping current: {}", e, current_task.project_id));
                    current_task.project_id.try_into().unwrap()
                }
            }
        } else {
            debug_log(&format!("No project specified, keeping current: {}", current_task.project_id));
            current_task.project_id.try_into().unwrap()
        };
        let updated_task = VikunjaTask {
            id: Some(task_id as u64),
            title: parsed.title.clone(),
            description: current_task.description,
            done: current_task.done,
            priority: parsed.priority.or(current_task.priority),
            due_date: parsed.due_date.or(current_task.due_date),
            project_id: project_id.try_into().unwrap(),
            labels: None,
            assignees: None,
        };
        debug_log(&format!("Updating task with project_id: {}, title: '{}'", project_id, updated_task.title));
        let updated_task = self.update_task(&updated_task).await?;
        debug_log(&format!("Task updated with ID: {:?}", updated_task.id));
        // Remove all existing labels, then add only those present in the edit line
        if let Some(existing_labels) = &current_task.labels {
            for label in existing_labels {
                if let Some(label_id) = label.id {
                    let _ = self.remove_label_from_task(task_id as u64, label_id).await;
                }
            }
        }
        for label_name in &parsed.labels {
            if let Ok(label) = self.ensure_label_exists(label_name).await {
                let _ = self.add_label_to_task(task_id as u64, label.id.unwrap()).await;
            }
        }
        if !parsed.assignees.is_empty() {
            if let Some(existing_assignees) = &current_task.assignees {
                for assignee in existing_assignees {
                    if let Some(user_id) = assignee.id {
                        let _ = self.remove_assignee_from_task(task_id as u64, user_id).await;
                    }
                }
            }
            for username in &parsed.assignees {
                if let Some(user) = self.find_user_by_username(username).await {
                    let _ = self.add_assignee_to_task(task_id as u64, user.id.unwrap()).await;
                }
            }
        }
        if let Some(_repeat) = &parsed.repeat_interval {
            // Implement repeat logic based on Vikunja's repeat API
        }
        self.get_task(task_id as u64).await
    }

    pub async fn update_task(&self, task: &VikunjaTask) -> ReqwestResult<VikunjaTask> {
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

    pub async fn ensure_label_exists(&self, label_name: &str) -> ReqwestResult<VikunjaLabel> {
        if let Ok(Some(label)) = self.find_label_by_name(label_name).await {
            return Ok(label);
        }
        self.create_label(label_name).await
    }

    pub async fn find_label_by_name(&self, label_name: &str) -> ReqwestResult<Option<VikunjaLabel>> {
        let url = format!("{}/api/v1/labels", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let labels: Vec<VikunjaLabel> = response.json().await?;
        Ok(labels.into_iter().find(|l| l.title.eq_ignore_ascii_case(label_name)))
    }

    pub async fn create_label(&self, label_name: &str) -> ReqwestResult<VikunjaLabel> {
        let url = format!("{}/api/v1/labels", self.base_url);
        let label = VikunjaLabel {
            id: None,
            title: label_name.to_string(),
            hex_color: None,
        };
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&label)
            .send()
            .await?;
        response.json().await
    }

    pub async fn add_label_to_task(&self, task_id: u64, label_id: u64) -> ReqwestResult<()> {
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

    pub async fn remove_label_from_task(&self, task_id: u64, label_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/labels/{}", self.base_url, task_id, label_id);
        let _response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        Ok(())
    }

    pub async fn add_assignee_to_task(&self, task_id: u64, user_id: u64) -> ReqwestResult<()> {
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

    pub async fn remove_assignee_from_task(&self, task_id: u64, user_id: u64) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/assignees/{}", self.base_url, task_id, user_id);
        let _response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_task(&self, task_id: i64) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        Ok(())
    }

    pub async fn update_task_completion(&self, task_id: i64, done: bool) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        let mut map = HashMap::new();
        map.insert("done", done);
        self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&map)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_tasks_with_projects(&self) -> Result<(
        Vec<crate::vikunja::models::Task>,
        std::collections::HashMap<i64, String>,
        std::collections::HashMap<i64, String>,
    ), reqwest::Error> {
        // Fetch projects
        let url = format!("{}/api/v1/projects", self.base_url);
        let projects_resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let projects: Vec<crate::vikunja::models::Project> = projects_resp.json().await?;
        // Build project_map and project_colors
        let mut project_map = std::collections::HashMap::new();
        let mut project_colors = std::collections::HashMap::new();
        for project in &projects {
            project_map.insert(project.id, project.title.clone());
            project_colors.insert(project.id, project.hex_color.clone());
        }
        // Fetch all tasks
        let url = format!("{}/api/v1/tasks/all", self.base_url);
        let tasks_resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let tasks: Vec<crate::vikunja::models::Task> = tasks_resp.json().await?;
        Ok((tasks, project_map, project_colors))
    }
}

// Make VikunjaClient fields public for setup_test_env.rs
impl crate::vikunja_client::VikunjaClient {
    pub fn base_url(&self) -> &str { &self.base_url }
    pub fn client(&self) -> &reqwest::Client { &self.client }
    pub fn auth_token(&self) -> &str { &self.auth_token }
}

// ... Project, Filter, User impls remain in their files ...
