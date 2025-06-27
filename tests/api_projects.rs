//! Integration tests for Vikunja projects API
use cria::vikunja_client::{VikunjaClient};

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
async fn test_find_or_get_project_id() {
    let base_url = get_env_var(&["VIKUNJA_URL", "VIKUNJA_API_URL"], "http://localhost:3456");
    let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    let client = VikunjaClient::new(base_url, token);

    // Try to find a project by name (replace with a real project name in your test instance)
    let project_name = get_env_var(&["VIKUNJA_PROJECT_NAME"], "Inbox");
    let id = client.find_or_get_project_id(&project_name).await.expect("find_or_get_project_id failed");
    assert!(id.is_some(), "Project '{}' not found", project_name);
}
