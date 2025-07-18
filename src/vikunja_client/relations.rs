use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::vikunja_client::VikunjaClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRelation {
    pub task_id: u64,
    pub other_task_id: u64,
    pub relation_kind: RelationKind,
    pub created_by: Option<crate::vikunja::models::User>,
    pub created: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RelationKind {
    Subtask,
    Parenttask,
    Related,
    Duplicateof,
    Duplicates,
    Blocking,
    Blocked,
    Precedes,
    Follows,
    Copiedfrom,
    Copiedto,
}

impl RelationKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationKind::Subtask => "Subtask of",
            RelationKind::Parenttask => "Parent of",
            RelationKind::Related => "Related to",
            RelationKind::Duplicateof => "Duplicate of",
            RelationKind::Duplicates => "Duplicates",
            RelationKind::Blocking => "Blocking",
            RelationKind::Blocked => "Blocked by",
            RelationKind::Precedes => "Precedes",
            RelationKind::Follows => "Follows",
            RelationKind::Copiedfrom => "Copied from",
            RelationKind::Copiedto => "Copied to",
        }
    }

    pub fn is_blocking_relation(&self) -> bool {
        matches!(self, RelationKind::Blocked | RelationKind::Blocking)
    }

    pub fn reverse(&self) -> RelationKind {
        match self {
            RelationKind::Subtask => RelationKind::Parenttask,
            RelationKind::Parenttask => RelationKind::Subtask,
            RelationKind::Related => RelationKind::Related,
            RelationKind::Duplicateof => RelationKind::Duplicates,
            RelationKind::Duplicates => RelationKind::Duplicateof,
            RelationKind::Blocking => RelationKind::Blocked,
            RelationKind::Blocked => RelationKind::Blocking,
            RelationKind::Precedes => RelationKind::Follows,
            RelationKind::Follows => RelationKind::Precedes,
            RelationKind::Copiedfrom => RelationKind::Copiedto,
            RelationKind::Copiedto => RelationKind::Copiedfrom,
        }
    }
}

#[derive(Debug, Serialize)]
struct CreateRelationRequest {
    other_task_id: u64,
    relation_kind: RelationKind,
}

impl VikunjaClient {
    /// Create a new task relation
    pub async fn create_task_relation(
        &self,
        task_id: u64,
        other_task_id: u64,
        relation_kind: RelationKind,
    ) -> Result<TaskRelation, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/tasks/{}/relations", self.base_url, task_id);
        
        let request = CreateRelationRequest {
            other_task_id,
            relation_kind,
        };

        let response = self.client
            .put(&url)
            .bearer_auth(&self.auth_token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let relation: TaskRelation = response.json().await?;
            Ok(relation)
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to create relation: {}", error_text).into())
        }
    }

    /// Delete a task relation
    pub async fn delete_task_relation(
        &self,
        task_id: u64,
        other_task_id: u64,
        relation_kind: RelationKind,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "{}/api/v1/tasks/{}/relations/{}/{}",
            self.base_url, task_id, relation_kind.to_string().to_lowercase(), other_task_id
        );

        let response = self.client
            .delete(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to delete relation: {}", error_text).into())
        }
    }

    /// Get all relations for a task
    pub async fn get_task_relations(
        &self,
        task_id: u64,
    ) -> Result<HashMap<String, Vec<crate::vikunja::models::Task>>, Box<dyn std::error::Error>> {
        let url = format!("{}/api/v1/tasks/{}", self.base_url, task_id);

        let response = self.client
            .get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().is_success() {
            let task: crate::vikunja::models::Task = response.json().await?;
            Ok(task.related_tasks.unwrap_or_default())
        } else {
            let error_text = response.text().await?;
            Err(format!("Failed to get task relations: {}", error_text).into())
        }
    }
}

impl ToString for RelationKind {
    fn to_string(&self) -> String {
        match self {
            RelationKind::Subtask => "subtask".to_string(),
            RelationKind::Parenttask => "parenttask".to_string(),
            RelationKind::Related => "related".to_string(),
            RelationKind::Duplicateof => "duplicateof".to_string(),
            RelationKind::Duplicates => "duplicates".to_string(),
            RelationKind::Blocking => "blocking".to_string(),
            RelationKind::Blocked => "blocked".to_string(),
            RelationKind::Precedes => "precedes".to_string(),
            RelationKind::Follows => "follows".to_string(),
            RelationKind::Copiedfrom => "copiedfrom".to_string(),
            RelationKind::Copiedto => "copiedto".to_string(),
        }
    }
}