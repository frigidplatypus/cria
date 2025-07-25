use cria::vikunja::models::{Task, Comment, User, Label};
use chrono::{DateTime, Utc};

/// Builder pattern for creating test Task instances with various configurations
pub struct TaskBuilder {
    task: Task,
    comment_id_counter: i64,
}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {
            task: Task {
                id: 1,
                title: "Test Task".to_string(),
                description: Some("".to_string()),
                done: false,
                done_at: Some("0001-01-01T00:00:00Z".to_string()),
                project_id: 1,
                labels: None,
                assignees: None,
                priority: Some(0),
                due_date: None,
                start_date: None,
                end_date: Some("0001-01-01T00:00:00Z".to_string()),
                created: Some("2025-01-01T00:00:00Z".to_string()),
                updated: Some("2025-01-01T00:00:00Z".to_string()),
                created_by: Some(User {
                    id: 1,
                    username: "test_user".to_string(),
                    name: Some("Test User".to_string()),
                    email: None,
                    created: Some("2025-01-01T00:00:00Z".to_string()),
                    updated: Some("2025-01-01T00:00:00Z".to_string()),
                }),
                percent_done: Some(0),
                is_favorite: false,
                position: Some(0),
                index: Some(1),
                identifier: Some("#1".to_string()),
                hex_color: Some("".to_string()),
                cover_image_attachment_id: Some(0),
                bucket_id: Some(0),
                buckets: None,
                attachments: None,
                comments: None,
                reactions: None,
                related_tasks: Some(std::collections::HashMap::new()),
                reminders: None,
                repeat_after: Some(0),
                repeat_mode: Some(0),
                subscription: None,
            },
            comment_id_counter: 1,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.task.id = id;
        self
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.task.title = title.to_string();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.task.description = Some(desc.to_string());
        self
    }

    pub fn with_empty_description(mut self) -> Self {
        self.task.description = Some("".to_string());
        self
    }

    pub fn with_no_description(mut self) -> Self {
        self.task.description = None;
        self
    }

    pub fn with_comment(mut self, author: &str, text: &str) -> Self {
        let comment = Comment {
            id: self.comment_id_counter,
            comment: Some(text.to_string()),
            author: Some(User {
                id: self.comment_id_counter,
                username: author.to_string(),
                name: Some(author.to_string()),
                email: None,
                created: Some("2025-01-01T00:00:00Z".to_string()),
                updated: Some("2025-01-01T00:00:00Z".to_string()),
            }),
            created: Some("2025-01-01T00:00:00Z".to_string()),
            updated: Some("2025-01-01T00:00:00Z".to_string()),
            reactions: None,
        };

        if self.task.comments.is_none() {
            self.task.comments = Some(vec![]);
        }
        self.task.comments.as_mut().unwrap().push(comment);
        self.comment_id_counter += 1;
        self
    }

    pub fn with_comment_no_author(mut self, text: &str) -> Self {
        let comment = Comment {
            id: self.comment_id_counter,
            comment: Some(text.to_string()),
            author: None,
            created: Some("2025-01-01T00:00:00Z".to_string()),
            updated: Some("2025-01-01T00:00:00Z".to_string()),
            reactions: None,
        };

        if self.task.comments.is_none() {
            self.task.comments = Some(vec![]);
        }
        self.task.comments.as_mut().unwrap().push(comment);
        self.comment_id_counter += 1;
        self
    }

    pub fn with_empty_comment(mut self, author: &str) -> Self {
        let comment = Comment {
            id: self.comment_id_counter,
            comment: Some("".to_string()),
            author: Some(User {
                id: self.comment_id_counter,
                username: author.to_string(),
                name: Some(author.to_string()),
                email: None,
                created: Some("2025-01-01T00:00:00Z".to_string()),
                updated: Some("2025-01-01T00:00:00Z".to_string()),
            }),
            created: Some("2025-01-01T00:00:00Z".to_string()),
            updated: Some("2025-01-01T00:00:00Z".to_string()),
            reactions: None,
        };

        if self.task.comments.is_none() {
            self.task.comments = Some(vec![]);
        }
        self.task.comments.as_mut().unwrap().push(comment);
        self.comment_id_counter += 1;
        self
    }

    pub fn with_project_id(mut self, project_id: i64) -> Self {
        self.task.project_id = project_id;
        self
    }

    pub fn build(self) -> Task {
        self.task
    }
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-built test scenarios for common URL testing patterns
pub struct TestScenarios;

impl TestScenarios {
    /// Task with same URL in description and comment (tests deduplication priority)
    pub fn duplicate_url_task() -> Task {
        TaskBuilder::new()
            .with_description("Visit https://example.com for more info")
            .with_comment("test_user", "Also check https://example.com")
            .build()
    }

    /// Task with HTML anchor link (tests HTML deduplication)  
    pub fn html_anchor_task() -> Task {
        TaskBuilder::new()
            .with_description(r#"<p><a target="_blank" href="https://site.com">https://site.com</a></p>"#)
            .build()
    }

    /// Task with multiple different URLs across description and comments
    pub fn multiple_urls_task() -> Task {
        TaskBuilder::new()
            .with_description("Check https://site1.com and https://site2.com")
            .with_comment("user1", "Also see https://site3.com")
            .with_comment("user2", "Don't forget https://site4.com")
            .build()
    }

    /// Task with mixed duplicate and unique URLs
    pub fn complex_deduplication_task() -> Task {
        TaskBuilder::new()
            .with_description("Visit https://example.com and https://unique1.com")
            .with_comment("user1", "Check https://example.com again")
            .with_comment("user2", "Also https://unique2.com and https://example.com")
            .build()
    }

    /// Task with no URLs anywhere
    pub fn no_urls_task() -> Task {
        TaskBuilder::new()
            .with_description("This task has no links")
            .with_comment("user", "Just plain text here")
            .build()
    }

    /// Task with edge case URLs
    pub fn edge_case_urls_task() -> Task {
        TaskBuilder::new()
            .with_description("Visit https://very-long-domain-name-that-exceeds-typical-limits.example.com/with/very/long/path")
            .with_comment("user", "Also http://localhost:8080/api/v1/test?param=value&other=123#fragment")
            .build()
    }

    /// Task with malformed URL attempts
    pub fn malformed_urls_task() -> Task {
        TaskBuilder::new()
            .with_description("Bad links: ftp://example.com ://broken.com http:// https://")
            .with_comment("user", "More bad: www.example.com example.com mailto:test@example.com")
            .build()
    }

    /// Task with no comments
    pub fn no_comments_task() -> Task {
        TaskBuilder::new()
            .with_description("Visit https://example.com")
            .build()
    }

    /// Task with empty comments list
    pub fn empty_comments_task() -> Task {
        let mut task = TaskBuilder::new()
            .with_description("Visit https://example.com")
            .build();
        task.comments = Some(vec![]);
        task
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_builder_basic() {
        let task = TaskBuilder::new().build();
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test Task");
        assert!(task.description.is_some());
    }

    #[test]
    fn test_task_builder_with_comment() {
        let task = TaskBuilder::new()
            .with_comment("author", "comment text")
            .build();
        
        assert!(task.comments.is_some());
        let comments = task.comments.unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].comment.as_ref().unwrap(), "comment text");
        assert_eq!(comments[0].author.as_ref().unwrap().username, "author");
    }

    #[test]
    fn test_multiple_comments() {
        let task = TaskBuilder::new()
            .with_comment("user1", "first comment")
            .with_comment("user2", "second comment")
            .build();
        
        let comments = task.comments.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].comment.as_ref().unwrap(), "first comment");
        assert_eq!(comments[1].comment.as_ref().unwrap(), "second comment");
    }

    #[test]
    fn test_scenarios() {
        let task = TestScenarios::duplicate_url_task();
        assert!(task.description.as_ref().unwrap().contains("https://example.com"));
        assert!(task.comments.is_some());
        
        let task = TestScenarios::no_urls_task();
        assert!(!task.description.as_ref().unwrap().contains("http"));
    }
}