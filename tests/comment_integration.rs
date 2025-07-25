use cria::tui::app::state::App;
use cria::vikunja::models::Task;
use cria::url_utils::extract_urls_from_task;

mod helpers;
use helpers::{TaskBuilder};
use helpers::task_builders::TestScenarios;

#[cfg(test)]
mod comment_integration_tests {
    use super::*;

    #[test]
    fn test_url_opening_uses_detailed_task_when_available() {
        // GIVEN: App with basic task (no comments) and cached detailed task (with comments)
        let mut app = App::default();
        
        // Basic task without comments (simulates initial task list)
        let basic_task = TaskBuilder::new()
            .with_id(123)
            .with_title("Test Task")
            .with_description("Visit https://description-url.com")
            .build(); // No comments by default
            
        // Detailed task with comments (simulates task fetched with get_task_detailed)
        let detailed_task = TaskBuilder::new()
            .with_id(123)
            .with_title("Test Task")
            .with_description("Visit https://description-url.com")
            .with_comment("user", "Check https://comment-url.com")
            .build();

        // Set up app state
        app.tasks = vec![basic_task];
        app.cache_detailed_task(detailed_task);
        app.selected_task_index = 0;

        // WHEN: Simulate the URL opening logic (what happens when user presses 'o')
        let basic_task = app.get_selected_task().unwrap();
        let task_to_use = app.get_detailed_task(basic_task.id).unwrap_or(basic_task);

        // THEN: Should use detailed task with comments
        assert!(task_to_use.comments.is_some(), "Should use detailed task with comments");
        assert_eq!(task_to_use.comments.as_ref().unwrap().len(), 1);
        
        // AND: URL extraction should find URLs from both description and comments
        let urls = extract_urls_from_task(task_to_use);
        assert_eq!(urls.len(), 2, "Should find URLs from both description and comments");
        
        let url_sources: std::collections::HashMap<String, String> = urls
            .into_iter()
            .map(|u| (u.url, u.source))
            .collect();
            
        assert_eq!(url_sources.get("https://description-url.com"), Some(&"Description".to_string()));
        assert_eq!(url_sources.get("https://comment-url.com"), Some(&"Comment by user".to_string()));
    }

    #[test]
    fn test_falls_back_to_basic_task_when_no_cache() {
        // GIVEN: App with basic task but NO cached detailed task
        let mut app = App::default();
        
        let basic_task = TaskBuilder::new()
            .with_id(456)
            .with_title("Basic Task")
            .with_description("Visit https://basic-url.com")
            .build(); // No comments

        app.tasks = vec![basic_task];
        app.selected_task_index = 0;
        // Note: NOT calling cache_detailed_task

        // WHEN: Simulate URL opening logic
        let basic_task = app.get_selected_task().unwrap();
        let task_to_use = app.get_detailed_task(basic_task.id).unwrap_or(basic_task);

        // THEN: Should fall back to basic task
        assert!(task_to_use.comments.is_none(), "Should fall back to basic task without comments");
        
        // AND: URL extraction should only find description URLs
        let urls = extract_urls_from_task(task_to_use);
        assert_eq!(urls.len(), 1, "Should only find URL from description");
        assert_eq!(urls[0].url, "https://basic-url.com");
        assert_eq!(urls[0].source, "Description");
    }

    #[test]
    fn test_detailed_task_cache_functionality() {
        // GIVEN: App instance
        let mut app = App::default();
        
        let detailed_task = TaskBuilder::new()
            .with_id(789)
            .with_title("Cached Task")
            .with_comment("author", "Cached comment")
            .build();

        // WHEN: Cache a detailed task
        app.cache_detailed_task(detailed_task.clone());

        // THEN: Should be able to retrieve it
        let retrieved = app.get_detailed_task(789);
        assert!(retrieved.is_some(), "Should be able to retrieve cached detailed task");
        
        let retrieved_task = retrieved.unwrap();
        assert_eq!(retrieved_task.id, 789);
        assert_eq!(retrieved_task.title, "Cached Task");
        assert!(retrieved_task.comments.is_some());
        assert_eq!(retrieved_task.comments.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_detailed_task_cache_overwrites_existing() {
        // GIVEN: App with an existing cached task
        let mut app = App::default();
        
        let first_task = TaskBuilder::new()
            .with_id(100)
            .with_title("First Version")
            .with_comment("user1", "First comment")
            .build();
            
        let second_task = TaskBuilder::new()
            .with_id(100) // Same ID
            .with_title("Second Version")
            .with_comment("user2", "Second comment")
            .build();

        // WHEN: Cache first task, then cache second task with same ID
        app.cache_detailed_task(first_task);
        app.cache_detailed_task(second_task);

        // THEN: Should retrieve the second (newer) version
        let retrieved = app.get_detailed_task(100).unwrap();
        assert_eq!(retrieved.title, "Second Version");
        assert_eq!(retrieved.comments.as_ref().unwrap()[0].comment.as_ref().unwrap(), "Second comment");
    }

    #[test]
    fn test_url_extraction_difference_basic_vs_detailed() {
        // GIVEN: Two versions of the same task - basic without comments, detailed with comments
        let basic_task = TaskBuilder::new()
            .with_id(200)
            .with_description("https://desc.com")
            .build(); // No comments
            
        let detailed_task = TaskBuilder::new()
            .with_id(200)
            .with_description("https://desc.com")
            .with_comment("user", "https://comment.com")
            .build();

        // WHEN: Extract URLs from both versions
        let basic_urls = extract_urls_from_task(&basic_task);
        let detailed_urls = extract_urls_from_task(&detailed_task);

        // THEN: Basic should have fewer URLs than detailed
        assert_eq!(basic_urls.len(), 1, "Basic task should only have description URL");
        assert_eq!(detailed_urls.len(), 2, "Detailed task should have both description and comment URLs");
        
        assert_eq!(basic_urls[0].url, "https://desc.com");
        assert_eq!(basic_urls[0].source, "Description");
        
        let detailed_sources: std::collections::HashMap<String, String> = detailed_urls
            .into_iter()
            .map(|u| (u.url, u.source))
            .collect();
            
        assert!(detailed_sources.contains_key("https://desc.com"));
        assert!(detailed_sources.contains_key("https://comment.com"));
    }

    #[test]
    fn test_multiple_tasks_in_cache() {
        // GIVEN: App with multiple cached detailed tasks
        let mut app = App::default();
        
        let task1 = TaskBuilder::new()
            .with_id(301)
            .with_title("Task 1")
            .with_comment("user1", "Comment 1")
            .build();
            
        let task2 = TaskBuilder::new()
            .with_id(302)
            .with_title("Task 2")
            .with_comment("user2", "Comment 2")
            .build();

        // WHEN: Cache multiple tasks
        app.cache_detailed_task(task1);
        app.cache_detailed_task(task2);

        // THEN: Should be able to retrieve each by ID
        let retrieved1 = app.get_detailed_task(301).unwrap();
        let retrieved2 = app.get_detailed_task(302).unwrap();
        
        assert_eq!(retrieved1.title, "Task 1");
        assert_eq!(retrieved2.title, "Task 2");
        
        // AND: Should not interfere with each other
        assert_eq!(retrieved1.comments.as_ref().unwrap()[0].comment.as_ref().unwrap(), "Comment 1");
        assert_eq!(retrieved2.comments.as_ref().unwrap()[0].comment.as_ref().unwrap(), "Comment 2");
    }

    #[test]
    fn test_nonexistent_detailed_task_returns_none() {
        // GIVEN: App with no cached tasks
        let app = App::default();

        // WHEN: Try to get a non-existent detailed task
        let result = app.get_detailed_task(999);

        // THEN: Should return None
        assert!(result.is_none(), "Non-existent detailed task should return None");
    }

    #[test]
    fn test_complete_workflow_simulation() {
        // GIVEN: Complete simulation of the URL opening workflow
        let mut app = App::default();
        
        // Set up task list (what user sees initially)
        let basic_task = TaskBuilder::new()
            .with_id(500)
            .with_title("Workflow Test")
            .with_description("Visit https://main-site.com")
            .build();
            
        app.tasks = vec![basic_task];
        app.selected_task_index = 0;

        // Simulate that detailed task was fetched and cached (happens in background)
        let detailed_task = TaskBuilder::new()
            .with_id(500)
            .with_title("Workflow Test")
            .with_description("Visit https://main-site.com")
            .with_comment("teammate", "Also check https://additional-site.com")
            .with_comment("boss", "Don't forget https://important-site.com")
            .build();
            
        app.cache_detailed_task(detailed_task);

        // WHEN: User presses 'o' key (simulated workflow)
        // Step 1: Get selected task (basic version)
        let basic_task = app.get_selected_task();
        assert!(basic_task.is_some(), "Should have selected task");
        
        // Step 2: Try to get detailed version, fall back to basic
        let task_to_use = app.get_detailed_task(basic_task.unwrap().id).unwrap_or(basic_task.unwrap());
        
        // Step 3: Extract URLs
        let urls = extract_urls_from_task(task_to_use);

        // THEN: Should find all URLs from detailed task
        assert_eq!(urls.len(), 3, "Complete workflow should find all URLs");
        
        let found_urls: std::collections::HashSet<String> = urls
            .iter()
            .map(|u| u.url.clone())
            .collect();
            
        assert!(found_urls.contains("https://main-site.com"));
        assert!(found_urls.contains("https://additional-site.com"));  
        assert!(found_urls.contains("https://important-site.com"));

        // AND: Should be ready to show URL modal
        assert!(!urls.is_empty(), "Should have URLs for modal");
    }
}

// Test the integration with test scenarios
#[cfg(test)]
mod scenario_integration_tests {
    use super::*;

    #[test]
    fn test_scenarios_with_app_integration() {
        // Test that our TestScenarios work properly with App integration
        let mut app = App::default();
        
        // Use a complex scenario
        let complex_task = TestScenarios::complex_deduplication_task();
        let task_id = complex_task.id;
        
        app.tasks = vec![TaskBuilder::new().with_id(task_id).build()]; // Basic version
        app.cache_detailed_task(complex_task); // Detailed version
        app.selected_task_index = 0;
        
        // Simulate workflow
        let basic_task = app.get_selected_task().unwrap();
        let task_to_use = app.get_detailed_task(basic_task.id).unwrap_or(basic_task);
        let urls = extract_urls_from_task(task_to_use);
        
        // Should work correctly with complex deduplication
        assert!(urls.len() >= 2, "Complex scenario should have multiple URLs");
    }
}