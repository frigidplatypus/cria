use regex::Regex;
use std::sync::OnceLock;

static URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_url_regex() -> &'static Regex {
    URL_REGEX.get_or_init(|| {
        // Improved URL regex: matches http(s) URLs, stops at common trailing punctuation and brackets
        Regex::new(r"https?://[^\s)>,;}]+").expect("Invalid URL regex")
    })
}

/// Extract all URLs from a given text
pub fn extract_urls(text: &str) -> Vec<String> {
    get_url_regex()
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// URL with context information about where it was found
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UrlWithContext {
    pub url: String,
    pub source: String, // "Description" or "Comment by {author}"
}

/// Extract URLs from task description and comments with context
pub fn extract_urls_from_task(task: &crate::vikunja::models::Task) -> Vec<UrlWithContext> {
    let mut urls = Vec::new();
    
    // Extract from task description
    if let Some(description) = &task.description {
        for url in extract_urls(description) {
            urls.push(UrlWithContext {
                url,
                source: "Description".to_string(),
            });
        }
    }
    
    // Extract from comments
    if let Some(comments) = &task.comments {
        for comment in comments {
            let comment_text = comment.comment.as_deref().unwrap_or("");
            for url in extract_urls(comment_text) {
                let author = comment.author.as_ref()
                    .map(|a| a.username.as_str())
                    .unwrap_or("Unknown");
                urls.push(UrlWithContext {
                    url,
                    source: format!("Comment by {}", author),
                });
            }
        }
    }
    
    urls
}

/// Open a URL using the system's default browser
pub fn open_url(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vikunja::models::{Task, Comment, User};

    fn create_test_task() -> Task {
        Task {
            id: 1,
            title: "Test Task".to_string(),
            description: None,
            done: false,
            done_at: None,
            project_id: 1,
            labels: None,
            assignees: None,
            priority: None,
            due_date: None,
            start_date: None,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: false,
            position: None,
            index: None,
            identifier: None,
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: None,
            buckets: None,
            attachments: None,
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: None,
            subscription: None,
        }
    }

    #[test]
    fn test_extract_urls() {
        let text = "Check out https://github.com/user/repo and http://example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://github.com/user/repo");
        assert_eq!(urls[1], "http://example.com");
    }

    #[test]
    fn test_no_urls() {
        let text = "This text has no URLs";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_url_with_query_params() {
        let text = "Visit https://example.com/path?param=value&other=123";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com/path?param=value&other=123");
    }

    #[test]
    fn test_urls_with_fragments() {
        let text = "Go to https://example.com/page#section and https://test.com/#top";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com/page#section".to_string()));
        assert!(urls.contains(&"https://test.com/#top".to_string()));
    }

    #[test]
    fn test_urls_in_markdown() {
        let text = "Check [this link](https://example.com) and https://direct.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"https://example.com".to_string()));
        assert!(urls.contains(&"https://direct.com".to_string()));
    }

    #[test]
    fn test_mixed_case_protocols() {
        let text = "HTTP://example.com and HTTPS://secure.com";
        let urls = extract_urls(text);
        // Our regex is case-sensitive for simplicity, but should handle common cases
        assert_eq!(urls.len(), 0); // These won't match due to case sensitivity
        
        let text_lowercase = "http://example.com and https://secure.com";
        let urls_lowercase = extract_urls(text_lowercase);
        assert_eq!(urls_lowercase.len(), 2);
    }

    #[test]
    fn test_urls_with_ports() {
        let text = "Visit http://localhost:3000 and https://example.com:8080/path";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"http://localhost:3000".to_string()));
        assert!(urls.contains(&"https://example.com:8080/path".to_string()));
    }

    #[test]
    fn test_urls_with_userinfo() {
        let text = "Connect to https://user:pass@example.com/secure";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://user:pass@example.com/secure");
    }

    #[test]
    fn test_extract_urls_from_task_with_description() {
        let mut task = create_test_task();
        task.description = Some("Check https://example.com for details".to_string());
        
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Description");
    }

    #[test]
    fn test_extract_urls_from_task_with_comments() {
        let mut task = create_test_task();
        
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            created: None,
            updated: None,
        };
        
        let comment = Comment {
            id: 1,
            author: Some(user),
            comment: Some("See https://github.com/project/repo".to_string()),
            created: None,
            updated: None,
            reactions: None,
        };
        
        task.comments = Some(vec![comment]);
        
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://github.com/project/repo");
        assert_eq!(urls[0].source, "Comment by testuser");
    }

    #[test]
    fn test_extract_urls_from_task_multiple_urls_in_description() {
        let mut task = create_test_task();
        task.description = Some("Check https://example.com and https://test.com for info".to_string());
        
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|u| u.url == "https://example.com" && u.source == "Description"));
        assert!(urls.iter().any(|u| u.url == "https://test.com" && u.source == "Description"));
    }

    #[test]
    fn test_extract_urls_from_task_empty_comment() {
        let mut task = create_test_task();
        
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            created: None,
            updated: None,
        };
        
        let comment = Comment {
            id: 1,
            author: Some(user),
            comment: None, // Empty comment
            created: None,
            updated: None,
            reactions: None,
        };
        
        task.comments = Some(vec![comment]);
        
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 0);
    }

    #[test]
    fn test_extract_urls_from_task_comment_without_author() {
        let mut task = create_test_task();
        
        let comment = Comment {
            id: 1,
            author: None,
            comment: Some("Anonymous comment with https://example.com".to_string()),
            created: None,
            updated: None,
            reactions: None,
        };
        
        task.comments = Some(vec![comment]);
        
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Comment by Unknown");
    }

    #[test]
    fn test_url_with_context_creation() {
        let url_ctx = UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Test source".to_string(),
        };
        
        assert_eq!(url_ctx.url, "https://example.com");
        assert_eq!(url_ctx.source, "Test source");
    }

    #[test]
    fn test_url_with_context_clone() {
        let original = UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Test source".to_string(),
        };
        
        let cloned = original.clone();
        assert_eq!(original.url, cloned.url);
        assert_eq!(original.source, cloned.source);
    }

    #[test]
    fn test_extract_urls_boundary_cases() {
        // URL at start of text
        let text = "https://example.com is a great site";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com");
        
        // URL at end of text
        let text = "Visit https://example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com");
        
        // URL is entire text
        let text = "https://example.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "https://example.com");
        
        // Multiple URLs separated by punctuation
        let text = "https://example.com,https://test.com;https://final.com";
        let urls = extract_urls(text);
        assert_eq!(urls.len(), 3);
    }

    // Note: We don't test open_url function here since it's system-dependent
    // and would require mocking the process execution
}
