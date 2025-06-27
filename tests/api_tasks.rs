//! Integration tests for Vikunja tasks API
use cria::vikunja_client::{VikunjaClient, tasks::VikunjaTask};

fn get_env_var(keys: &[&str], default: &str) -> String {
    for &key in keys {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                return val;
            }
        }
    }
    default.to_string()
}

#[tokio::test]
async fn test_create_and_delete_task() {
    let base_url = get_env_var(&["VIKUNJA_URL", "VIKUNJA_API_URL"], "http://localhost:3456");
    let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    let project_id = get_env_var(&["VIKUNJA_PROJECT_ID"], "1").parse().unwrap_or(1);
    let client = VikunjaClient::new(base_url, token);

    // Create a task
    let task = VikunjaTask {
        id: None,
        title: "Test Task from API".to_string(),
        description: Some("Created by integration test".to_string()),
        done: Some(false),
        priority: Some(1),
        due_date: None,
        project_id,
        labels: None,
        assignees: None,
    };
    let created = client.create_task(&task).await.expect("create_task failed");
    assert_eq!(created.title, task.title);

    // Delete the task
    client.delete_task(created.id.unwrap() as i64).await.expect("delete_task failed");
}
