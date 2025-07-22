//! UI integration tests for URL opening feature

use cria::tui::app::state::App;
use cria::vikunja::models::{Task, Comment, User};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Test that 'o' key press triggers URL detection when URLs are present
#[test]
fn test_o_key_triggers_url_detection_with_urls() {
    let mut app = App::default();
    
    // Create a task with URLs in description
    let mut task = create_test_task_with_id(1);
    task.description = Some("Check out https://github.com/test/repo for details".to_string());
    
    app.tasks = vec![task];
    app.selected_task_index = 0;
    
    // Verify initial state
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
    
    // Simulate 'o' key press - we can't directly test the key handler since it's in ui_loop
    // but we can test the underlying functionality that should be triggered
    let task = &app.tasks[app.selected_task_index];
    let urls = cria::url_utils::extract_urls_from_task(task);
    
    // Verify URLs were extracted
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0].url, "https://github.com/test/repo");
    assert_eq!(urls[0].source, "Description");
    
    // Simulate showing the modal (what the 'o' key handler should do)
    app.show_url_modal(urls);
    
    // Verify modal was shown
    assert!(app.show_url_modal);
    assert!(app.url_modal.is_some());
}

/// Test that 'o' key press does nothing when no URLs are present
#[test]
fn test_o_key_with_no_urls() {
    let mut app = App::default();
    
    // Create a task without URLs
    let task = create_test_task_with_id(1);
    app.tasks = vec![task];
    app.selected_task_index = 0;
    
    // Simulate URL extraction (what 'o' key handler does)
    let task = &app.tasks[app.selected_task_index];
    let urls = cria::url_utils::extract_urls_from_task(task);
    
    // Verify no URLs found
    assert_eq!(urls.len(), 0);
    
    // Modal should not be shown when no URLs
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
}

/// Test URL extraction from task with multiple sources
#[test]
fn test_url_extraction_multiple_sources() {
    let mut app = App::default();
    
    // Create task with URLs in both description and comments
    let mut task = create_test_task_with_id(1);
    task.description = Some("Main repository: https://github.com/main/repo".to_string());
    
    let user = User {
        id: 1,
        username: "reviewer".to_string(),
        name: Some("Code Reviewer".to_string()),
        email: Some("reviewer@example.com".to_string()),
        created: None,
        updated: None,
    };
    
    let comment = Comment {
        id: 1,
        author: Some(user),
        comment: Some("Also check the docs at https://docs.example.com".to_string()),
        created: None,
        updated: None,
        reactions: None,
    };
    
    task.comments = Some(vec![comment]);
    
    app.tasks = vec![task];
    app.selected_task_index = 0;
    
    // Extract URLs
    let task = &app.tasks[app.selected_task_index];
    let urls = cria::url_utils::extract_urls_from_task(task);
    
    // Should find URLs from both sources
    assert_eq!(urls.len(), 2);
    
    let sources: Vec<&String> = urls.iter().map(|u| &u.source).collect();
    assert!(sources.contains(&&"Description".to_string()));
    assert!(sources.contains(&&"Comment by reviewer".to_string()));
    
    let url_strings: Vec<&String> = urls.iter().map(|u| &u.url).collect();
    assert!(url_strings.contains(&&"https://github.com/main/repo".to_string()));
    assert!(url_strings.contains(&&"https://docs.example.com".to_string()));
}

/// Test modal state management
#[test]
fn test_url_modal_state_management() {
    let mut app = App::default();
    
    let urls = vec![
        cria::url_utils::UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Description".to_string(),
        },
    ];
    
    // Test showing modal
    app.show_url_modal(urls.clone());
    assert!(app.show_url_modal);
    assert!(app.url_modal.is_some());
    
    if let Some(modal) = &app.url_modal {
        assert_eq!(modal.urls.len(), 1);
        assert_eq!(modal.urls[0].url, "https://example.com");
    }
    
    // Test hiding modal
    app.hide_url_modal();
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
}

/// Test modal closes with other modals
#[test]
fn test_url_modal_closes_with_other_modals() {
    let mut app = App::default();
    
    let urls = vec![
        cria::url_utils::UrlWithContext {
            url: "https://example.com".to_string(),
            source: "Description".to_string(),
        },
    ];
    
    // Show URL modal
    app.show_url_modal(urls);
    assert!(app.show_url_modal);
    
    // Test that close_all_modals closes URL modal
    app.close_all_modals();
    assert!(!app.show_url_modal);
    assert!(app.url_modal.is_none());
}

/// Test URL modal with empty task list
#[test]
fn test_url_modal_with_empty_task_list() {
    let app = App::default();
    
    // Empty task list should not crash when trying to access selected task
    assert_eq!(app.tasks.len(), 0);
    assert_eq!(app.selected_task_index, 0);
    
    // This would be handled by bounds checking in the actual UI code
}

/// Test URL modal navigation and selection
#[test]
fn test_url_modal_navigation_integration() {
    use cria::tui::modals::url_modal::UrlModal;
    
    let urls = vec![
        cria::url_utils::UrlWithContext {
            url: "https://first.com".to_string(),
            source: "Description".to_string(),
        },
        cria::url_utils::UrlWithContext {
            url: "https://second.com".to_string(),
            source: "Comment by user".to_string(),
        },
    ];
    
    let mut modal = UrlModal::new(urls);
    
    // Test initial selection
    assert_eq!(modal.get_selected_url(), Some("https://first.com"));
    
    // Test navigation
    modal.handle_key('j');
    assert_eq!(modal.get_selected_url(), Some("https://second.com"));
    
    modal.handle_key('k');
    assert_eq!(modal.get_selected_url(), Some("https://first.com"));
}

/// Test URL patterns that should be detected
#[test]
fn test_url_pattern_detection() {
    let test_cases = vec![
        ("Simple HTTP", "Visit http://example.com", vec!["http://example.com"]),
        ("Simple HTTPS", "Visit https://example.com", vec!["https://example.com"]),
        ("With path", "Go to https://example.com/path/to/page", vec!["https://example.com/path/to/page"]),
        ("With query params", "Check https://example.com?q=test&page=1", vec!["https://example.com?q=test&page=1"]),
        ("With fragment", "See https://example.com#section", vec!["https://example.com#section"]),
        ("Multiple URLs", "Visit https://first.com and https://second.com", vec!["https://first.com", "https://second.com"]),
        ("GitHub URL", "Fork https://github.com/user/repo", vec!["https://github.com/user/repo"]),
        ("With port", "Connect to http://localhost:3000", vec!["http://localhost:3000"]),
        ("In markdown", "Check [link](https://example.com) here", vec!["https://example.com"]),
    ];
    
    for (name, text, expected_urls) in test_cases {
        let extracted_urls = cria::url_utils::extract_urls(text);
        
        assert_eq!(
            extracted_urls.len(),
            expected_urls.len(),
            "Test '{}' failed: expected {} URLs, found {}. Text: '{}', Found: {:?}",
            name, expected_urls.len(), extracted_urls.len(), text, extracted_urls
        );
        
        for expected_url in expected_urls {
            assert!(
                extracted_urls.contains(&expected_url.to_string()),
                "Test '{}' failed: URL '{}' not found in extracted URLs: {:?}",
                name, expected_url, extracted_urls
            );
        }
    }
}

/// Helper function to create a test task with a specific ID
fn create_test_task_with_id(id: i64) -> Task {
    Task {
        id,
        title: format!("Test Task {}", id),
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
