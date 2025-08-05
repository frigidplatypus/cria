use reqwest::Client;
use std::path::Path;
use tokio::fs;
use crate::vikunja::models::Attachment;

pub struct AttachmentClient {
    client: Client,
    base_url: String,
    auth_token: String,
}

impl AttachmentClient {
    pub fn new(client: Client, base_url: String, auth_token: String) -> Self {
        Self {
            client,
            base_url,
            auth_token,
        }
    }

    /// Get attachments for a specific task
    pub async fn get_task_attachments(&self, task_id: i64) -> Result<Vec<Attachment>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v1/tasks/{}/attachments", self.base_url, task_id);
        
        crate::debug::debug_log(&format!("Fetching attachments for task {}: {}", task_id, url));
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let response_text = response.text().await?;
            crate::debug::debug_log(&format!("Attachments response: {}", response_text));
            
            match serde_json::from_str::<Vec<Attachment>>(&response_text) {
                Ok(attachments) => {
                    crate::debug::debug_log(&format!("Found {} attachments", attachments.len()));
                    Ok(attachments)
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse attachments response: {}. Response: {}", e, response_text);
                    crate::debug::debug_log(&error_msg);
                    Err(error_msg.into())
                }
            }
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            let error_msg = format!("Failed to get attachments: {} - {}", status, error_text);
            crate::debug::debug_log(&error_msg);
            Err(error_msg.into())
        }
    }

    /// Upload a file attachment to a task (see Vikunja API docs)
    pub async fn upload_attachment(
        &self,
        task_id: i64,
        file_path: &Path,
    ) -> Result<Attachment, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Construct URL
        let url = format!("{}/api/v1/tasks/{}/attachments", self.base_url, task_id);
        crate::debug::debug_log(&format!("[upload_attachment] URL = {}", url));

        // Step 2: Read file
        crate::debug::debug_log(&format!("[upload_attachment] Reading file {}", file_path.display()));
        let file_content = fs::read(file_path).await?;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?;

        // Step 3: Build multipart form
        crate::debug::debug_log(&format!("[upload_attachment] Building multipart form"));
        let part = reqwest::multipart::Part::bytes(file_content.clone())
            .file_name(file_name.to_string());
        let form = reqwest::multipart::Form::new().part("files", part);

        // Step 4: Send request
        crate::debug::debug_log("[upload_attachment] Sending PUT request");
        let response = self.client
            .put(&url)
            .bearer_auth(&self.auth_token)
            .multipart(form)
            .send()
            .await?;

        // Step 5: Handle response
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        crate::debug::debug_log(&format!("[upload_attachment] Status: {}, Body: {}", status, body));

        if status.is_success() {
            // Parse returned list or fetch fresh list
            let attachments: Vec<Attachment> = match serde_json::from_str::<Vec<Attachment>>(&body) {
                Ok(arr) if !arr.is_empty() => arr,
                _ => {
                    crate::debug::debug_log("[upload_attachment] Fetching attachments after upload");
                    self.get_task_attachments(task_id).await?
                }
            };
            let attachment = attachments.into_iter()
                .last()
                .ok_or_else(|| "No attachments found after upload")?;
            crate::debug::debug_log(&format!("[upload_attachment] Uploaded ID {}", attachment.id));
            Ok(attachment)
        } else {
            Err(format!("Attachment upload failed ({}): {}", status, body).into())
        }
    }

    /// Download an attachment to a local file
    pub async fn download_attachment(
        &self,
        attachment: &Attachment,
        download_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(_) = &attachment.file {
            let url = format!(
                "{}/api/v1/tasks/{}/attachments/{}?download",
                self.base_url, attachment.task_id, attachment.id
            );
            crate::debug::debug_log(&format!("[download_attachment] GET {}", url));

            let response = self.client
                .get(&url)
                .bearer_auth(&self.auth_token)
                .send()
                .await?;

            crate::debug::debug_log(&format!("[download_attachment] Status {}", response.status()));
            if response.status().is_success() {
                let bytes = response.bytes().await?;
                crate::debug::debug_log(&format!(
                    "[download_attachment] Writing {} bytes to {:?}",
                    bytes.len(),
                    download_path
                ));
                fs::write(download_path, bytes).await?;
                crate::debug::debug_log("[download_attachment] File saved");
                Ok(())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                crate::debug::debug_log(&format!(
                    "[download_attachment] Error {}: {}",
                    status, body
                ));
                Err(format!("Failed to download attachment: {} - {}", status, body).into())
            }
        } else {
            Err("Attachment has no file data".into())
        }
    }

    /// Remove an attachment from a task
    pub async fn remove_attachment(
        &self,
        task_id: i64,
        attachment_id: i64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v1/tasks/{}/attachments/{}",
            self.base_url, task_id, attachment_id
        );
        let response = self.client
            .delete(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to remove attachment: {}", response.status()).into())
        }
    }

    /// Get attachment metadata for a task
    pub async fn get_attachment(
        &self,
        task_id: i64,
        attachment_id: i64,
    ) -> Result<Attachment, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/api/v1/tasks/{}/attachments/{}",
            self.base_url, task_id, attachment_id
        );
        let response = self.client
            .get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(format!("Failed to get attachment: {}", response.status()).into())
        }
    }
}

/// Helper function to format file size in human-readable format
pub fn format_file_size(size_bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = size_bytes as f64;
    
    if size >= GB {
        format!("{:.1} GB", size / GB)
    } else if size >= MB {
        format!("{:.1} MB", size / MB)
    } else if size >= KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{} B", size_bytes)
    }
}

/// Helper function to get file extension from filename
pub fn get_file_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension()?.to_str()
}

/// Helper function to check if file is an image based on extension
pub fn is_image_file(filename: &str) -> bool {
    if let Some(ext) = get_file_extension(filename) {
        let ext_lower = ext.to_lowercase();
        matches!(ext_lower.as_str(), 
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" | "tif"
        )
    } else {
        false
    }
}

/// Helper function to get file icon based on file type
pub fn get_file_icon(filename: &str) -> &'static str {
    if is_image_file(filename) {
        "🖼️"
    } else if let Some(ext) = get_file_extension(filename) {
        let ext_lower = ext.to_lowercase();
        match ext_lower.as_str() {
            "pdf" => "📄",
            "txt" => "📄",
            "md" => "📝",
            "zip" | "rar" | "7z" | "tar" | "gz" => "📦",
            "mp3" | "wav" | "flac" | "ogg" => "🎵",
            "mp4" | "avi" | "mov" | "mkv" => "🎬",
            "py" | "js" | "rs" | "go" | "java" | "cpp" | "c" => "💻",
            "html" | "css" | "xml" | "json" => "🌐",
            _ => "📎"
        }
    } else {
        "📎"
    }
}
