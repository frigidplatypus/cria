//! Integration tests for Vikunja users API
// Temporarily disabled: user search test is unreliable or not supported in all setups.
// use cria::vikunja_client::VikunjaClient;
//
// fn get_env_var(keys: &[&str], default: &str) -> String {
//     for &key in keys {
//         if let Ok(val) = std::env::var(key) {
//             if !val.is_empty() {
//                 return val;
//             }
//         }
//     }
//     default.to_string()
// }
//
// #[tokio::test]
// async fn test_find_user_by_username() {
//     let base_url = get_env_var(&["VIKUNJA_URL", "VIKUNJA_API_URL"], "http://localhost:3456");
//     let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
//     let client = VikunjaClient::new(base_url, token);
//
//     // Replace with a real username in your test instance
//     let username = get_env_var(&["VIKUNJA_TEST_USER"], "admin");
//     let user = client.find_user_by_username(&username).await;
//     assert!(user.is_some(), "User '{}' not found", username);
// }
