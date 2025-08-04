// Task-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use reqwest::Result as ReqwestResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::debug::debug_log;
use crate::vikunja_client::VikunjaUser;
use serde_json;

#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaTask {
    pub id: Option<u64>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub done: Option<bool>,
    pub priority: Option<u8>,
    pub due_date: Option<DateTime<Utc>>,
    pub start_date: Option<DateTime<Utc>>,
    pub project_id: u64,
    pub labels: Option<Vec<VikunjaLabel>>,
    pub assignees: Option<Vec<VikunjaUser>>,
    pub is_favorite: Option<bool>,
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
        default_project_id: i64,
    ) -> ReqwestResult<VikunjaTask> {
        debug_log(&format!("Parsing magic text: '{}'", magic_text));
        let parsed = self.parser.parse(magic_text);
        debug_log(&format!("Parsed task - title: '{}', labels: {:?}, project: {:?}", 
                 parsed.title, parsed.labels, parsed.project));
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
                    default_project_id
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project '{}': {}. Using default: {}.", project_name, e, default_project_id));
                    default_project_id
                }
            }
        } else {
            debug_log(&format!("No project specified, using default: {}.", default_project_id));
            default_project_id
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
            start_date: None,
            project_id: project_id as u64,
            labels: None,
            assignees: None,
            is_favorite: Some(false),
        };

        debug_log(&format!("Creating task with project_id: {}, title: '{}'", project_id, task.title));
        let created_task = self.create_task(&task).await?;
        debug_log(&format!("Task created with ID: {:?}", created_task.id));
        let task_id = created_task.id.unwrap();

        // Step 3: Add labels
        debug_log(&format!("Step 3: Adding {} labels to task {}", parsed.labels.len(), task_id));
        for label_name in &parsed.labels {
            debug_log(&format!("Processing label: '{}'", label_name));
            match self.ensure_label_exists(label_name).await {
                Ok(label) => {
                    debug_log(&format!("Label '{}' exists/created with ID: {:?}", label_name, label.id));
                    match self.add_label_to_task(task_id, label.id.unwrap()).await {
                        Ok(_) => debug_log(&format!("Successfully added label '{}' to task {}", label_name, task_id)),
                        Err(e) => debug_log(&format!("Failed to add label '{}' to task {}: {}", label_name, task_id, e)),
                    }
                }
                Err(e) => debug_log(&format!("Failed to ensure label '{}' exists: {}", label_name, e)),
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

        // Return the updated task (with proper refresh to ensure it's in the next fetch)
        debug_log(&format!("SUCCESS: Task created successfully! ID: {:?}, Title: '{}'", created_task.id, created_task.title));
        
        // Wait a moment to ensure the server has processed everything
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(created_task)
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
                    debug_log(&format!("create_task API error response ({}): {} characters", status, error_text.len()));
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

    pub async fn get_task_detailed(&self, task_id: u64) -> Result<crate::vikunja::models::Task, reqwest::Error> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
            
        let mut task: crate::vikunja::models::Task = response.json().await?;
        
        // Now fetch comments separately and merge them into the task
        match self.get_task_comments(task_id).await {
            Ok(comments) => {
                task.comments = Some(comments);
            }
            Err(_e) => {
                // Keep comments as None if fetching fails
            }
        }
        
        Ok(task)
    }

    pub async fn update_task_with_magic(
        &self,
        task_id: i64,
        magic_text: &str,
    ) -> ReqwestResult<VikunjaTask> {
        debug_log(&format!("Updating task {} with magic text: '{}'", task_id, magic_text));
        let parsed = self.parser.parse(magic_text);
        debug_log(&format!("Parsed task - title: '{}', labels: {:?}, project: {:?}", 
                 parsed.title, parsed.labels, parsed.project));
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
                    current_task.project_id as i64
                }
                Err(e) => {
                    debug_log(&format!("Error looking up project: {}, keeping current: {}", e, current_task.project_id));
                    current_task.project_id as i64
                }
            }
        } else {
            debug_log(&format!("No project specified, keeping current: {}", current_task.project_id));
            current_task.project_id as i64
        };
        let updated_task = VikunjaTask {
            id: Some(task_id as u64),
            title: parsed.title.clone(),
            description: current_task.description,
            done: current_task.done,
            priority: parsed.priority.or(current_task.priority),
            due_date: parsed.due_date.or(current_task.due_date),
            start_date: current_task.start_date,
            project_id: project_id as u64,
            labels: None,
            assignees: None,
            is_favorite: current_task.is_favorite,
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
        // Log JSON payload for debugging
        let json_str = serde_json::to_string(task).unwrap_or_default();
        debug_log(&format!("update_task JSON payload: {}", json_str));
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
                    debug_log(&format!("update_task API error response ({}): {} characters", status, error_text.len()));
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

    #[allow(dead_code)]
    pub async fn update_task_with_form_data(
        &self,
        task_id: i64,
        title: &str,
        description: Option<&str>,
        priority: Option<i32>,
        project_id: i64,
        is_favorite: bool,
    ) -> ReqwestResult<VikunjaTask> {
        debug_log(&format!("Updating task {} with form data - title: '{}', project_id: {}, favorite: {}", 
                 task_id, title, project_id, is_favorite));

        let task = VikunjaTask {
            id: Some(task_id as u64),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            done: None, // Don't change done status in form edit
            priority: priority.map(|p| p as u8),
            due_date: None, // TODO: Parse due_date from form
            start_date: None,
            project_id: project_id as u64,
            labels: None, // TODO: Handle labels from form
            assignees: None, // TODO: Handle assignees from form
            is_favorite: Some(is_favorite),
        };

        self.update_task(&task).await
    }
    
    pub async fn update_task_from_form(
        &self,
        task_id: i64,
        title: &str,
        description: &str,
        due_date: Option<&str>,
        start_date: Option<&str>,
        priority: Option<i32>,
        project_id: i64,
        label_ids: &[i64],
        assignee_ids: &[i64],
        is_favorite: bool,
        comment: Option<&str>,
    ) -> Result<crate::vikunja::models::Task, Box<dyn Error>> {
        debug_log(&format!("Updating task {} from form - title: '{}', project_id: {}, favorite: {}", 
                 task_id, title, project_id, is_favorite));

        // Parse dates
        let due_date_parsed = if let Some(date_str) = due_date {
            if !date_str.trim().is_empty() {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(23, 59, 59).unwrap().and_utc())
            } else {
                None
            }
        } else {
            None
        };

        let start_date_parsed = if let Some(date_str) = start_date {
            if !date_str.trim().is_empty() {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .ok()
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
            } else {
                None
            }
        } else {
            None
        };

        // Create task object for update
        // Prepare HTML description body
        let description_html = if description.trim().is_empty() {
            None
        } else {
            Some(format!("<p>{}</p>", description.trim()))
        };
        let task = VikunjaTask {
            id: Some(task_id as u64),
            title: title.to_string(),
            description: description_html,
            done: None, // Don't change done status in form edit
            priority: priority.map(|p| p as u8),
            due_date: due_date_parsed,
            start_date: start_date_parsed,
            project_id: project_id as u64,
            labels: None, // Will be set separately
            assignees: None, // Will be set separately
            is_favorite: Some(is_favorite),
        };

        // Log JSON payload for debugging description
        match serde_json::to_string(&task) {
            Ok(json) => debug_log(&format!("update_task_from_form JSON: {}", json)),
            Err(e) => debug_log(&format!("Failed to serialize task JSON: {}", e)),
        }
        // Update the basic task
        let _updated_task = self.update_task(&task).await?;
        
        // Handle labels - first remove all existing labels, then add new ones
        if let Err(e) = self.clear_task_labels(task_id as u64).await {
            debug_log(&format!("Warning: Failed to clear labels for task {}: {}", task_id, e));
        }
        
        for &label_id in label_ids {
            if let Err(e) = self.add_label_to_task(task_id as u64, label_id as u64).await {
                debug_log(&format!("Warning: Failed to add label {} to task {}: {}", label_id, task_id, e));
            }
        }

        // Handle assignees - similar approach
        if let Err(e) = self.clear_task_assignees(task_id as u64).await {
            debug_log(&format!("Warning: Failed to clear assignees for task {}: {}", task_id, e));
        }
        
        for &assignee_id in assignee_ids {
            if let Err(e) = self.add_assignee_to_task(task_id as u64, assignee_id as u64).await {
                debug_log(&format!("Warning: Failed to add assignee {} to task {}: {}", assignee_id, task_id, e));
            }
        }


        // Handle comment
        if let Some(comment_text) = comment {
            if !comment_text.trim().is_empty() {
                if let Err(e) = self.add_comment_to_task(task_id as u64, comment_text).await {
                    debug_log(&format!("Warning: Failed to add comment to task {}: {}", task_id, e));
                }
            }
        }

        // Get the final updated task
        match self.get_task(task_id as u64).await {
            Ok(vikunja_task) => {
                let model = crate::vikunja::models::Task::from_vikunja_task(vikunja_task);
                debug_log(&format!("update_task_from_form returned task: {:?}", model));
                Ok(model)
            }
            Err(e) => Err(Box::new(e) as Box<dyn Error>)
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
        // Fetch all tasks using comprehensive method
        debug_log("Starting comprehensive task fetch after task creation...");
        
        let tasks = self.get_all_tasks_comprehensive().await?;
        
        Ok((tasks, project_map, project_colors))
    }

    pub async fn get_all_tasks_comprehensive(&self) -> Result<Vec<crate::vikunja::models::Task>, reqwest::Error> {
        debug_log("Starting comprehensive task fetch...");
        
        // Method 1: Try paginated /api/v1/tasks/all
        match self.get_tasks_paginated().await {
            Ok(tasks) => {
                debug_log(&format!("Method 1 (paginated): Success, got {} tasks", tasks.len()));
                return Ok(tasks);
            }
            Err(e) => {
                debug_log(&format!("Method 1 (paginated) failed: {}", e));
            }
        }
        
        // Method 2: Try simple /api/v1/tasks/all with high limit
        match self.get_tasks_simple_with_limit().await {
            Ok(tasks) => {
                debug_log(&format!("Method 2 (simple with limit): Success, got {} tasks", tasks.len()));
                return Ok(tasks);
            }
            Err(e) => {
                debug_log(&format!("Method 2 (simple with limit) failed: {}", e));
            }
        }
        
        // Method 3: Aggregate tasks from all projects
        match self.get_tasks_from_all_projects().await {
            Ok(tasks) => {
                debug_log(&format!("Method 3 (from all projects): Success, got {} tasks", tasks.len()));
                return Ok(tasks);
            }
            Err(e) => {
                debug_log(&format!("Method 3 (from all projects) failed: {}", e));
                return Err(e);
            }
        }
    }
    
    async fn get_tasks_paginated(&self) -> Result<Vec<crate::vikunja::models::Task>, reqwest::Error> {
        let mut all_tasks = Vec::new();
        let mut page = 1;
        let per_page = 250; // Use a reasonable page size
        
        debug_log("Starting paginated task fetch...");
        
        loop {
            // Use comprehensive parameters to get all tasks (done and not done)
            let url = format!("{}/api/v1/tasks/all?page={}&per_page={}&sort_by=id&order_by=desc&filter_include_nulls=true", 
                             self.base_url, page, per_page);
            
            debug_log(&format!("Fetching page {} with URL: {}", page, url));
            
            let tasks_resp = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.auth_token))
                .send()
                .await?;
                
            let status = tasks_resp.status();
            debug_log(&format!("Page {} response status: {}", page, status));
            
            if !status.is_success() {
                let error_text = tasks_resp.text().await.unwrap_or_default();
                debug_log(&format!("Page {} failed with error: {} characters", page, error_text.len()));
                break;
            }
                
            let page_tasks: Vec<crate::vikunja::models::Task> = tasks_resp.json().await?;
            let page_count = page_tasks.len();
            
            debug_log(&format!("Page {} returned {} tasks", page, page_count));
            
            // Check if this page contains task 147
            if page_tasks.iter().any(|t| t.id == 147) {
                debug_log(&format!("✓ Found task 147 on page {}", page));
            }
            
            all_tasks.extend(page_tasks);
            
            // If we got fewer tasks than requested, we've reached the end
            if page_count < per_page {
                debug_log(&format!("Reached end of pagination on page {} (got {} < {})", page, page_count, per_page));
                break;
            }
            
            page += 1;
            if page > 100 { // Safety check to prevent infinite loops
                debug_log("Hit pagination safety limit of 100 pages");
                break;
            }
        }
        
        debug_log(&format!("Pagination complete: {} total tasks across {} pages", all_tasks.len(), page - 1));
        Ok(all_tasks)
    }
    
    async fn get_tasks_simple_with_limit(&self) -> Result<Vec<crate::vikunja::models::Task>, reqwest::Error> {
        // Try with a very high limit and include nulls to get everything
        let url = format!("{}/api/v1/tasks/all?per_page=10000&filter_include_nulls=true&sort_by=id&order_by=desc", self.base_url);
        
        debug_log(&format!("Trying simple fetch with high limit: {}", url));
        
        let tasks_resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
            
        let status = tasks_resp.status();
        debug_log(&format!("Simple fetch response status: {}", status));
        
        if !status.is_success() {
            let error_text = tasks_resp.text().await.unwrap_or_default();
            debug_log(&format!("Simple fetch failed: {} characters", error_text.len()));
            // Force a proper reqwest error by making a bad request
            let _bad_response = self.client.get("http://localhost:1/invalid").send().await;
            return Err(_bad_response.unwrap_err());
        }
        
        let tasks: Vec<crate::vikunja::models::Task> = tasks_resp.json().await?;
        debug_log(&format!("Simple fetch returned {} tasks", tasks.len()));
        
        if tasks.iter().any(|t| t.id == 147) {
            debug_log("✓ Found task 147 in simple fetch");
        } else {
            debug_log("✗ Task 147 not found in simple fetch");
        }
        
        Ok(tasks)
    }
    
    async fn get_tasks_from_all_projects(&self) -> Result<Vec<crate::vikunja::models::Task>, reqwest::Error> {
        debug_log("Fetching tasks from all projects individually...");
        
        // Get all projects first
        let projects_url = format!("{}/api/v1/projects", self.base_url);
        let projects_resp = self.client
            .get(&projects_url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        let projects: Vec<crate::vikunja::models::Project> = projects_resp.json().await?;
        
        debug_log(&format!("Found {} projects to fetch tasks from", projects.len()));
        
        let mut all_tasks = Vec::new();
        
        // Get tasks from each project with comprehensive parameters
        for project in projects {
            let tasks_url = format!("{}/api/v1/projects/{}/tasks?per_page=10000&filter_include_nulls=true&sort_by=id&order_by=desc", 
                                   self.base_url, project.id);
            debug_log(&format!("Fetching tasks from project {} ({}): {}", project.id, project.title, tasks_url));
            
            match self.client
                .get(&tasks_url)
                .header("Authorization", format!("Bearer {}", self.auth_token))
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        match resp.json::<Vec<crate::vikunja::models::Task>>().await {
                            Ok(mut project_tasks) => {
                                debug_log(&format!("Project {} returned {} tasks", project.id, project_tasks.len()));
                                
                                // Check if this project contains task 147
                                if project_tasks.iter().any(|t| t.id == 147) {
                                    debug_log(&format!("✓ Found task 147 in project {} ({})", project.id, project.title));
                                }
                                
                                all_tasks.append(&mut project_tasks);
                            }
                            Err(e) => {
                                debug_log(&format!("Failed to parse tasks from project {}: {}", project.id, e));
                                continue;
                            }
                        }
                    } else {
                        debug_log(&format!("Project {} returned status: {}", project.id, status));
                        continue;
                    }
                }
                Err(e) => {
                    debug_log(&format!("Failed to fetch from project {}: {}", project.id, e));
                    continue;
                }
            }
        }
        
        debug_log(&format!("Project aggregation complete: {} total tasks", all_tasks.len()));
        Ok(all_tasks)
    }

    // Helper methods for form editing
    pub async fn clear_task_labels(&self, task_id: u64) -> ReqwestResult<()> {
        // Get current task to find existing labels
        let task = self.get_task(task_id).await?;
        if let Some(labels) = task.labels {
            for label in labels {
                if let Some(label_id) = label.id {
                    let _ = self.remove_label_from_task(task_id, label_id).await;
                }
            }
        }
        Ok(())
    }
    
    pub async fn clear_task_assignees(&self, task_id: u64) -> ReqwestResult<()> {
        // Get current task to find existing assignees
        let task = self.get_task(task_id).await?;
        if let Some(assignees) = task.assignees {
            for assignee in assignees {
                if let Some(assignee_id) = assignee.id {
                    let _ = self.remove_assignee_from_task(task_id, assignee_id).await;
                }
            }
        }
        Ok(())
    }

    pub async fn set_task_favorite(&self, task_id: u64, is_favorite: bool) -> ReqwestResult<()> {
        // Update just the favorite status by making a task update with minimal data
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);
        
        // Create a minimal task update that only changes the favorite status
        let task_update = serde_json::json!({
            "is_favorite": is_favorite
        });
        
        debug_log(&format!("Setting task {} favorite status to: {}", task_id, is_favorite));
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&task_update)
            .send()
            .await;
            
        match response {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    debug_log(&format!("Successfully updated favorite status for task {}", task_id));
                    Ok(())
                } else {
                    let error_text = resp.text().await.unwrap_or_else(|_| "Failed to read error response".to_string());
                    debug_log(&format!("API error updating favorite status ({}): {} characters", status, error_text.len()));
                    // Create an error similar to how other methods do it
                    let fake_response = self.client.get("http://invalid-url-that-will-fail").send().await;
                    Err(fake_response.unwrap_err())
                }
            }
            Err(e) => {
                debug_log(&format!("Request error updating favorite status: {}", e));
                Err(e)
            }
        }
    }

    pub async fn add_comment_to_task(&self, task_id: u64, comment: &str) -> ReqwestResult<()> {
        let url = format!("{}/api/v1/tasks/{}/comments", self.base_url, task_id);
        let comment_data = serde_json::json!({
            "comment": comment
        });
        self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .json(&comment_data)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_task_comments(&self, task_id: u64) -> Result<Vec<crate::vikunja::models::Comment>, reqwest::Error> {
        crate::debug::debug_log(&format!("Fetching comments for task {}", task_id));
        let url = format!("{}/api/v1/tasks/{}/comments", self.base_url, task_id);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        
        let comments: Vec<crate::vikunja::models::Comment> = response.json().await?;
        crate::debug::debug_log(&format!("Fetched {} comments for task {}", comments.len(), task_id));
        Ok(comments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_omit_none_description() {
        // When description is None, JSON should not include the field
        let task = VikunjaTask {
            id: Some(1),
            title: "Test".to_string(),
            description: None,
            done: None,
            priority: None,
            due_date: None,
            start_date: None,
            project_id: 0,
            labels: None,
            assignees: None,
            is_favorite: None,
        };
        let json = serde_json::to_value(&task).unwrap();
        assert!(!json.as_object().unwrap().contains_key("description"));
    }

    #[test]
    fn test_serialize_some_description() {
        // When description is Some, JSON should include the field
        let desc = "hello".to_string();
        let task = VikunjaTask {
            id: Some(2),
            title: "DescTest".to_string(),
            description: Some(desc.clone()),
            done: None,
            priority: None,
            due_date: None,
            start_date: None,
            project_id: 0,
            labels: None,
            assignees: None,
            is_favorite: None,
        };
        let json = serde_json::to_value(&task).unwrap();
        let value = json.get("description").unwrap().as_str().unwrap();
        assert_eq!(value, desc);
    }

    #[test]
    fn test_description_html_wrapping_logic() {
        // Replicate the HTML wrapping logic from update_task_from_form
        fn wrap(desc: &str) -> Option<String> {
            if desc.trim().is_empty() {
                None
            } else {
                Some(format!("<p>{}</p>", desc.trim()))
            }
        }
        assert_eq!(wrap(""), None);
        assert_eq!(wrap("foo"), Some("<p>foo</p>".to_string()));
        assert_eq!(wrap(" foo  "), Some("<p>foo</p>".to_string()));
    }
}
