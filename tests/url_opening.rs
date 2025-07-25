//! Integration tests for URL opening feature

use cria::tui::app::state::App;
use cria::tui::modals::url_modal::UrlModal;
use cria::url_utils::{extract_urls, extract_urls_from_task, UrlWithContext};
use cria::vikunja::models::{Task, Comment, User};

#[test]
fn test_extract_urls_basic() {
    let text = "Check out https://github.com/user/repo and http://example.com";
    let urls = extract_urls(text);
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0], "https://github.com/user/repo");
    assert_eq!(urls[1], "http://example.com");
}

#[test]
fn test_extract_urls_no_urls() {
    let text = "This text has no URLs";
    let urls = extract_urls(text);
    assert_eq!(urls.len(), 0);
}

#[test]
fn test_extract_urls_with_query_params() {
    let text = "Visit https://example.com/path?param=value&other=123";
    let urls = extract_urls(text);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://example.com/path?param=value&other=123");
}

#[test]
fn test_extract_urls_mixed_protocols() {
    let text = "HTTP: http://example.com and HTTPS: https://secure.example.com";
    let urls = extract_urls(text);
    assert_eq!(urls.len(), 2);
    assert!(urls.contains(&"http://example.com".to_string()));
    assert!(urls.contains(&"https://secure.example.com".to_string()));
}

#[test]
fn test_extract_urls_with_punctuation() {
    let text = "Check https://example.com, https://test.com! Also https://final.com.";
    let urls = extract_urls(text);
    assert_eq!(urls.len(), 3);
    // URLs should stop at punctuation
    assert!(urls.iter().any(|url| url.starts_with("https://example.com")));
    assert!(urls.iter().any(|url| url.starts_with("https://test.com")));
    assert!(urls.iter().any(|url| url.starts_with("https://final.com")));
}

#[test]
fn test_extract_urls_from_task_description_only() {
    let mut task = create_test_task();
    task.description = Some("Task description with https://example.com link".to_string());
    
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].url, "https://example.com");
    assert_eq!(urls[0].source, "Description");
}

#[test]
fn test_extract_urls_from_task_comments_only() {
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
        comment: Some("Check this link: https://github.com/test/repo".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    task.comments = Some(vec![comment]);
    
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].url, "https://github.com/test/repo");
    assert_eq!(urls[0].source, "Comment by testuser");
}

#[test]
fn test_extract_urls_from_task_both_description_and_comments() {
    let mut task = create_test_task();
    task.description = Some("Description with https://example.com".to_string());
    
    let user = User {
        id: 1,
        username: "commentuser".to_string(),
        name: Some("Comment User".to_string()),
        email: Some("comment@example.com".to_string()),
        created: None,
        updated: None,
    };
    
    let comment = Comment {
        id: 1,
        author: Some(user),
        comment: Some("Comment with https://github.com/comment/link".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    task.comments = Some(vec![comment]);
    
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 2);
    
    // Check that we have both sources
    let sources: Vec<&String> = urls.iter().map(|u| &u.source).collect();
    assert!(sources.contains(&&"Description".to_string()));
    assert!(sources.contains(&&"Comment by commentuser".to_string()));
}

#[test]
fn test_extract_urls_from_task_multiple_comments() {
    let mut task = create_test_task();
    
    let user1 = User {
        id: 1,
        username: "user1".to_string(),
        name: Some("User One".to_string()),
        email: Some("user1@example.com".to_string()),
        created: None,
        updated: None,
    };
    
    let user2 = User {
        id: 2,
        username: "user2".to_string(),
        name: Some("User Two".to_string()),
        email: Some("user2@example.com".to_string()),
        created: None,
        updated: None,
    };
    
    let comment1 = Comment {
        id: 1,
        author: Some(user1),
        comment: Some("First comment with https://first.com".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    let comment2 = Comment {
        id: 2,
        author: Some(user2),
        comment: Some("Second comment with https://second.com".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    task.comments = Some(vec![comment1, comment2]);
    
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 2);
    
    // Check that we have both users in sources
    let sources: Vec<&String> = urls.iter().map(|u| &u.source).collect();
    assert!(sources.contains(&&"Comment by user1".to_string()));
    assert!(sources.contains(&&"Comment by user2".to_string()));
}

#[test]
fn test_extract_urls_from_task_no_urls() {
    let task = create_test_task();
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 0);
}

#[test]
fn test_extract_urls_from_task_comment_without_author() {
    let mut task = create_test_task();
    
    let comment = Comment {
        id: 1,
        author: None,
        comment: Some("Anonymous comment with https://anonymous.com".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    task.comments = Some(vec![comment]);
    
    let urls = extract_urls_from_task(&task);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].url, "https://anonymous.com");
    assert_eq!(urls[0].source, "Comment by Unknown");
}

#[test]
fn test_url_modal_creation() {
    let urls = vec![
        UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Description".to_string(),
        },
        UrlWithContext {
            url: "https://github.com/test/repo".to_string(),
            source: "Comment by user".to_string(),
        },
    ];
    
    let modal = UrlModal::new(urls.clone());
    assert_eq!(modal.urls.len(), 2);
    assert_eq!(modal.selected_index, 0);
    assert_eq!(modal.urls, urls);
}

#[test]
fn test_url_modal_navigation() {
    let urls = vec![
        UrlWithContext {
            url: "https://example1.com".to_string(),
            source: "Description".to_string(),
        },
        UrlWithContext {
            url: "https://example2.com".to_string(),
            source: "Comment".to_string(),
        },
        UrlWithContext {
            url: "https://example3.com".to_string(),
            source: "Comment".to_string(),
        },
    ];
    
    let mut modal = UrlModal::new(urls);
    
    // Test moving down
    modal.handle_key('j');
    assert_eq!(modal.selected_index, 1);
    
    modal.handle_key('j');
    assert_eq!(modal.selected_index, 2);
    
    // Test wrapping at bottom
    modal.handle_key('j');
    assert_eq!(modal.selected_index, 0);
    
    // Test moving up
    modal.handle_key('k');
    assert_eq!(modal.selected_index, 2);
    
    modal.handle_key('k');
    assert_eq!(modal.selected_index, 1);
}

#[test]
fn test_url_modal_arrow_key_navigation() {
    let urls = vec![
        UrlWithContext {
            url: "https://example1.com".to_string(),
            source: "Description".to_string(),
        },
        UrlWithContext {
            url: "https://example2.com".to_string(),
            source: "Comment".to_string(),
        },
    ];
    
    let mut modal = UrlModal::new(urls);
    
    // Test arrow keys
    modal.handle_key('j');
    assert_eq!(modal.selected_index, 1);

    modal.handle_key('k');
    assert_eq!(modal.selected_index, 0);
}

#[test]
fn test_url_modal_get_selected_url() {
    let urls = vec![
        UrlWithContext {
            url: "https://example1.com".to_string(),
            source: "Description".to_string(),
        },
        UrlWithContext {
            url: "https://example2.com".to_string(),
            source: "Comment".to_string(),
        },
    ];
    
    let mut modal = UrlModal::new(urls);
    
    // Test getting selected URL
    assert_eq!(modal.get_selected_url(), Some("https://example1.com"));
    
    modal.handle_key('j');
    assert_eq!(modal.get_selected_url(), Some("https://example2.com"));
}

#[test]
fn test_url_modal_empty_urls() {
    let modal = UrlModal::new(vec![]);
    assert_eq!(modal.urls.len(), 0);
    assert_eq!(modal.selected_index, 0);
    assert_eq!(modal.get_selected_url(), None);
}

#[test]
fn test_app_url_modal_integration() {
    let mut app = App::default();
    
    let urls = vec![
        UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Description".to_string(),
        },
    ];
    
    // Test showing URL modal
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
    
    app.show_url_modal(urls.clone());
    assert!(app.show_url_modal);
    assert!(app.url_modal.is_some());
    
    if let Some(modal) = &app.url_modal {
        assert_eq!(modal.urls, urls);
    }
    
    // Test hiding URL modal
    app.hide_url_modal();
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
}

#[test]
fn test_app_close_all_modals_includes_url_modal() {
    let mut app = App::default();
    
    let urls = vec![
        UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Description".to_string(),
        },
    ];
    
    app.show_url_modal(urls);
    assert!(app.show_url_modal);
    assert!(app.url_modal.is_some());
    
    // Test that close_all_modals closes URL modal
    app.close_all_modals();
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
}

// Helper function to create a basic test task
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
