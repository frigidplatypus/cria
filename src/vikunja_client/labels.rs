use reqwest::Result as ReqwestResult;
use crate::vikunja_client::tasks::VikunjaLabel;

impl super::VikunjaClient {
    pub async fn get_all_labels(&self) -> ReqwestResult<Vec<VikunjaLabel>> {
        let url = format!("{}/api/v1/labels", self.base_url);
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;
        response.json().await
    }
}
