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

    /// Upload a file attachment to a task
    pub async fn upload_attachment(&self, task_id: i64, file_path: &Path) -> Result<Attachment, Box<dyn std::error::Error + Send + Sync>> {
        // Try different possible upload endpoints and methods
        let attempts = vec![
            ("PUT", format!("{}/api/v1/tasks/{}/attachments", self.base_url, task_id)),
            ("POST", format!("{}/api/v1/tasks/{}/attachments", self.base_url, task_id)),
            ("POST", format!("{}/api/v1/attachments", self.base_url)),
        ];
        
        crate::debug::debug_log(&format!("Uploading file to task {}: {}", task_id, file_path.display()));
        
        // Read the file
        let file_content = fs::read(file_path).await?;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?;

        // Try each endpoint and method
        for (method, url) in attempts {
            crate::debug::debug_log(&format!("Trying {} {}: {}", method, url, file_name));
            
            // Create multipart form with file for this attempt
            let form = reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(file_content.clone())
                    .file_name(file_name.to_string()));
            
            let response = if method == "PUT" {
                self.client
                    .put(&url)
                    .bearer_auth(&self.auth_token)
                    .multipart(form)
                    .send()
                    .await
            } else {
                self.client
                    .post(&url)
                    .bearer_auth(&self.auth_token)
                    .multipart(form)
                    .send()
                    .await
            };

            match response {
                Ok(response) => {
                    let status = response.status();
                    crate::debug::debug_log(&format!("Response status: {} for {} {}", status, method, url));
                    
                    if status.is_success() {
                        // First, let's see what the raw response looks like
                        let response_text = response.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
                        crate::debug::debug_log(&format!("Raw API response: {}", response_text));
                        
                        // Try to parse as JSON
                        match serde_json::from_str::<Attachment>(&response_text) {
                            Ok(attachment) => {
                                crate::debug::debug_log(&format!("Upload successful: attachment ID {}", attachment.id));
                                return Ok(attachment);
                            }
                            Err(e) => {
                                let error_msg = format!("Failed to parse attachment response: {}. Response: {}", e, response_text);
                                crate::debug::debug_log(&error_msg);
                                // Continue to next attempt
                            }
                        }
                    } else {
                        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                        let error_msg = format!("Upload failed with status {}: {}", status, error_text);
                        crate::debug::debug_log(&error_msg);
                        // Continue to next attempt
                    }
                }
                Err(e) => {
                    crate::debug::debug_log(&format!("Request failed for {} {}: {}", method, url, e));
                    // Continue to next attempt
                }
            }
        }
        
        // If we get here, all attempts failed
        Err("All upload attempts failed".into())
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