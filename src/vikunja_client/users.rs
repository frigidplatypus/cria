// User-related API functions for Vikunja
// ...will be filled in from vikunja_client.rs...

use serde::{Deserialize, Serialize};

// --- User-related types and functions ---
// VikunjaUser, user-related impls and functions
#[derive(Debug, Serialize, Deserialize)]
pub struct VikunjaUser {
    pub id: Option<u64>,
    pub username: String,
    pub name: Option<String>,
}

impl super::VikunjaClient {
    pub async fn find_user_by_username(&self, username: &str) -> Option<VikunjaUser> {
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
    // Add other user-related methods here as needed
}

// --- User-related API impls ---
// All user-related methods from VikunjaClient impl go here...
