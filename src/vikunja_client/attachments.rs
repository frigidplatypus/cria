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

    /// Upload a file attachment to a task (see Vikunja API docs: https://try.vikunja.io/api/v1/docs#tag/task/paths/~1tasks~1%7Bid%7D~1attachments/put)
    pub async fn upload_attachment(&self, task_id: i64, file_path: &Path) -> Result<Attachment, Box<dyn std::error::Error + Send + Sync>> {
        // Vikunja API docs: PUT /api/v1/tasks/{task_id}/attachments
        // Step 1: Construct the upload URL
        let url = format!("{}/api/v1/tasks/{}/attachments", self.base_url, task_id);
        crate::debug::debug_log(&format!("[upload_attachment] Step 1: URL = {}", url));

        // Step 2: Read file from disk
        crate::debug::debug_log(&format!("[upload_attachment] Step 2: Reading file {}", file_path.display()));
        let file_content = fs::read(file_path).await?;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?;
        // Step 3: Create multipart form (field 'file' with filename)
        crate::debug::debug_log(&format!("[upload_attachment] Step 3: Building multipart form with filename {}", file_name));
        let part = reqwest::multipart::Part::bytes(file_content.clone())
            .file_name(file_name.to_string());
        // Use 'files' as the form field per Vikunja docs
        let form = reqwest::multipart::Form::new()
            .part("files", part);

        // Step 4: Send PUT request with multipart form to upload attachment
        crate::debug::debug_log(&format!("[upload_attachment] Step 4: Sending request"));
        let response = self.client
            .put(&url)
            .bearer_auth(&self.auth_token)
            .multipart(form)
            .send()
            .await?;

        // Step 5: Parse response status and body
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        crate::debug::debug_log(&format!("[upload_attachment] Step 5: Response status {}", status));
        crate::debug::debug_log(&format!("[upload_attachment] Step 5: Response body {}", text));

        if status.is_success() {
            // Attempt to parse attachments from response
            let attachments: Vec<Attachment> = match serde_json::from_str::<Vec<Attachment>>(&text) {
                Ok(arr) if !arr.is_empty() => {
                    crate::debug::debug_log(&format!("Upload response parsed {} attachments", arr.len()));
                    arr
                }
                _ => {
                    // Response body not usable; fetch actual attachments list
                    crate::debug::debug_log("Upload response empty or invalid, fetching attachments list");
                    self.get_task_attachments(task_id).await?
                }
            };
            // Return the most recent attachment
            let attachment = attachments.into_iter().last()
                .ok_or_else(|| format!("No attachments found after upload"))?;
            crate::debug::debug_log(&format!("Upload successful: attachment ID {}", attachment.id));
            return Ok(attachment);
         } else {
             Err(format!("Attachment upload failed ({}): {}", status, text).into())
         }
    }

    /// Download an attachment to a local file
    pub async fn download_attachment(&self, attachment: &Attachment, download_path: &Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(file) = &attachment.file {
            let url = format!("{}/api/v1/attachments/{}/download", self.base_url, file.id);
            
            let response = self.client
                .get(&url)
                .bearer_auth(&self.auth_token)
                .send()
                .await?;

            if response.status().is_success() {
                let bytes = response.bytes().await?;
                fs::write(download_path, bytes).await?;
                Ok(())
            } else {
                Err(format!("Failed to download attachment: {}", response.status()).into())
            }
        } else {
            Err("Attachment has no file data".into())
        }
    }

    /// Remove an attachment from a task
    pub async fn remove_attachment(&self, attachment_id: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v1/attachments/{}", self.base_url, attachment_id);
        
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

    /// Get attachment metadata
    pub async fn get_attachment(&self, attachment_id: i64) -> Result<Attachment, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/v1/attachments/{}", self.base_url, attachment_id);
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().is_success() {
            let attachment: Attachment = response.json().await?;
            Ok(attachment)
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
            "doc" | "docx" => "📝",
            "xls" | "xlsx" => "📊",
            "ppt" | "pptx" => "📈",
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