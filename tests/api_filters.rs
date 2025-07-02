//! Integration tests for Vikunja filters API
use cria::vikunja_client::VikunjaClient;

mod common;

#[tokio::test]
async fn test_get_saved_filters() {
    if common::should_skip_integration_test() {
        println!("Skipping integration test - no API server available or SKIP_INTEGRATION_TESTS set");
        return;
    }
    
    let base_url = common::get_env_var(&["VIKUNJA_URL", "VIKUNJA_API_URL"], "http://localhost:3456");
    let token = common::get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    let client = VikunjaClient::new(base_url, token);

    let filters = client.get_saved_filters().await.expect("get_saved_filters failed");
    // Should not error, may be empty
    println!("Filters: {:?}", filters);
}
