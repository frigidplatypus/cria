//! Integration tests for Vikunja filters API
use cria::vikunja_client::VikunjaClient;

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
async fn test_get_saved_filters() {
    let base_url = get_env_var(&["VIKUNJA_URL", "VIKUNJA_API_URL"], "http://localhost:3456");
    let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    let client = VikunjaClient::new(base_url, token);

    let filters = client.get_saved_filters().await.expect("get_saved_filters failed");
    // Should not error, may be empty
    println!("Filters: {:?}", filters);
}
