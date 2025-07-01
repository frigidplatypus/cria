// Task-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use reqwest::Result as ReqwestResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::debug::debug_log;
use crate::vikunja_client::VikunjaUser;

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
        
        // First, debug what the API endpoints are returning
        let _ = self.debug_api_endpoints().await;
        
        let tasks = self.get_all_tasks_comprehensive().await?;
        
        // Check for task 147 specifically
        if let Some(task_147) = tasks.iter().find(|t| t.id == 147) {
            debug_log(&format!("✓ Found task 147: '{}'", task_147.title));
        } else {
            debug_log("✗ Task 147 not found in fetched tasks");
            // Show last few task IDs
            let mut recent_ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();
            recent_ids.sort();
            recent_ids.reverse();
            debug_log(&format!("Most recent task IDs: {:?}", recent_ids.iter().take(10).collect::<Vec<_>>()));
        }
        
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
                debug_log(&format!("Page {} failed with error: {}", page, error_text));
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
            debug_log(&format!("Simple fetch failed: {}", error_text));
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

    pub async fn debug_api_endpoints(&self) -> Result<(), reqwest::Error> {
        debug_log("=== API ENDPOINT DEBUGGING ===");
        
        // Test basic /api/v1/tasks/all
        let url1 = format!("{}/api/v1/tasks/all", self.base_url);
        debug_log(&format!("Testing: {}", url1));
        match self.client.get(&url1).header("Authorization", format!("Bearer {}", self.auth_token)).send().await {
            Ok(resp) => {
                let headers = resp.headers().clone();
                match resp.json::<Vec<crate::vikunja::models::Task>>().await {
                    Ok(tasks) => {
                        debug_log(&format!("✓ Basic /tasks/all returned {} tasks", tasks.len()));
                        debug_log(&format!("  Headers: {:?}", headers.get("x-pagination-total-pages")));
                        debug_log(&format!("  Pagination: total={:?}, current={:?}", 
                                  headers.get("x-pagination-total"), headers.get("x-pagination-current-page")));
                    }
                    Err(e) => debug_log(&format!("✗ Basic /tasks/all JSON parse error: {}", e))
                }
            }
            Err(e) => debug_log(&format!("✗ Basic /tasks/all request failed: {}", e))
        }
        
        // Test with proper per_page parameter and include nulls
        let url2 = format!("{}/api/v1/tasks/all?per_page=1000&filter_include_nulls=true", self.base_url);
        debug_log(&format!("Testing: {}", url2));
        match self.client.get(&url2).header("Authorization", format!("Bearer {}", self.auth_token)).send().await {
            Ok(resp) => {
                let headers = resp.headers().clone();
                match resp.json::<Vec<crate::vikunja::models::Task>>().await {
                    Ok(tasks) => {
                        debug_log(&format!("✓ With per_page=1000&filter_include_nulls=true returned {} tasks", tasks.len()));
                        debug_log(&format!("  Pagination: total={:?}, current={:?}, total_pages={:?}", 
                                  headers.get("x-pagination-total"), 
                                  headers.get("x-pagination-current-page"),
                                  headers.get("x-pagination-total-pages")));
                        
                        // Check if task 147 is in this result
                        if tasks.iter().any(|t| t.id == 147) {
                            debug_log("✓ Task 147 found in per_page=1000 result!");
                        } else {
                            debug_log("✗ Task 147 not found in per_page=1000 result");
                        }
                    }
                    Err(e) => debug_log(&format!("✗ With per_page=1000 JSON parse error: {}", e))
                }
            }
            Err(e) => debug_log(&format!("✗ With per_page=1000 request failed: {}", e))
        }
        
        // Test pagination explicitly
        let url3 = format!("{}/api/v1/tasks/all?page=1&per_page=100&filter_include_nulls=true", self.base_url);
        debug_log(&format!("Testing: {}", url3));
        match self.client.get(&url3).header("Authorization", format!("Bearer {}", self.auth_token)).send().await {
            Ok(resp) => {
                let headers = resp.headers().clone();
                match resp.json::<Vec<crate::vikunja::models::Task>>().await {
                    Ok(tasks) => {
                        debug_log(&format!("✓ Paginated (page=1, per_page=100) returned {} tasks", tasks.len()));
                        debug_log(&format!("  Pagination headers: {:?}", 
                                  vec![("total", headers.get("x-pagination-total")),
                                       ("current_page", headers.get("x-pagination-current-page")),
                                       ("total_pages", headers.get("x-pagination-total-pages")),
                                       ("per_page", headers.get("x-pagination-per-page"))]));
                    }
                    Err(e) => debug_log(&format!("✗ Paginated JSON parse error: {}", e))
                }
            }
            Err(e) => debug_log(&format!("✗ Paginated request failed: {}", e))
        }
        
        // Test direct task fetch for task 147
        let url4 = format!("{}/api/v1/tasks/147", self.base_url);
        debug_log(&format!("Testing direct fetch: {}", url4));
        match self.client.get(&url4).header("Authorization", format!("Bearer {}", self.auth_token)).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    match resp.json::<crate::vikunja::models::Task>().await {
                        Ok(task) => debug_log(&format!("✓ Direct fetch task 147: '{}' (project_id: {}, done: {})", 
                                                      task.title, task.project_id, task.done)),
                        Err(e) => debug_log(&format!("✗ Direct fetch task 147 JSON parse error: {}", e))
                    }
                } else {
                    debug_log(&format!("✗ Direct fetch task 147 failed with status: {}", status));
                }
            }
            Err(e) => debug_log(&format!("✗ Direct fetch task 147 request failed: {}", e))
        }
        
        debug_log("=== END API DEBUGGING ===");
        Ok(())
    }

    // ...existing code...
}

// Make VikunjaClient fields public for setup_test_env.rs
#[allow(dead_code)]
impl crate::vikunja_client::VikunjaClient {
    pub fn base_url(&self) -> &str { &self.base_url }
    pub fn client(&self) -> &reqwest::Client { &self.client }
    pub fn auth_token(&self) -> &str { &self.auth_token }
}

// ... Project, Filter, User impls remain in their files ...
