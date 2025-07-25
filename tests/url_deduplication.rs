use cria::url_utils::{extract_urls_from_task, UrlWithContext};
use cria::vikunja::models::{Task, Comment, User};

mod helpers;
use helpers::{TaskBuilder};
use helpers::task_builders::TestScenarios;

#[cfg(test)]
mod deduplication_tests {
    use super::*;

    #[test]
    fn test_description_wins_over_comment() {
        // GIVEN: Same URL appears in both description and comment
        let task = TaskBuilder::new()
            .with_description("Visit https://example.com for more info")
            .with_comment("test_user", "Also check https://example.com")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Only one URL should be found with Description source preferred
        assert_eq!(urls.len(), 1, "Expected exactly 1 unique URL after deduplication");
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Description", "Description source should take priority over comment");
    }

    #[test]
    fn test_multiple_comments_first_wins() {
        // GIVEN: Same URL appears in multiple comments (no description URL)
        let task = TaskBuilder::new()
            .with_empty_description()
            .with_comment("user1", "Check https://example.com first")
            .with_comment("user2", "Also see https://example.com again")
            .with_comment("user3", "And https://example.com one more time")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Only one URL with first comment's author
        assert_eq!(urls.len(), 1, "Expected exactly 1 unique URL from multiple comments");
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Comment by user1", "First comment should win when multiple comments have same URL");
    }

    #[test]
    fn test_html_anchor_deduplication() {
        // GIVEN: HTML anchor link where URL appears twice (the specific bug we fixed)
        let task = TaskBuilder::new()
            .with_description(r#"<p><a target="_blank" rel="noopener noreferrer nofollow" href="https://site.com">https://site.com</a></p>"#)
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // DEBUG: Print what URLs were found
        println!("Found {} URLs:", urls.len());
        for (i, url_ctx) in urls.iter().enumerate() {
            println!("  {}: {} (source: {})", i, url_ctx.url, url_ctx.source);
        }

        // THEN: Only one URL despite appearing twice in HTML
        assert_eq!(urls.len(), 1, "HTML anchor link should not create duplicate URLs");
        assert_eq!(urls[0].url, "https://site.com");
        assert_eq!(urls[0].source, "Description");
    }

    #[test]
    fn test_complex_deduplication_scenario() {
        // GIVEN: Multiple URLs with some duplicates across description and comments
        let task = TaskBuilder::new()
            .with_description("Visit https://example.com and https://unique1.com")
            .with_comment("user1", "Check https://example.com again and https://unique2.com")
            .with_comment("user2", "Also https://unique2.com and https://unique3.com")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should have 4 unique URLs with correct sources
        assert_eq!(urls.len(), 4, "Expected 4 unique URLs in complex scenario");
        
        // Convert to a map for easier assertion
        let url_map: std::collections::HashMap<String, String> = urls
            .into_iter()
            .map(|u| (u.url, u.source))
            .collect();

        // Verify each URL has correct source
        assert_eq!(url_map.get("https://example.com"), Some(&"Description".to_string()), 
                   "example.com should come from Description (priority over comment)");
        assert_eq!(url_map.get("https://unique1.com"), Some(&"Description".to_string()),
                   "unique1.com should come from Description");
        assert_eq!(url_map.get("https://unique2.com"), Some(&"Comment by user1".to_string()),
                   "unique2.com should come from first comment (priority over second comment)");
        assert_eq!(url_map.get("https://unique3.com"), Some(&"Comment by user2".to_string()),
                   "unique3.com should come from user2 comment");
    }

    #[test]
    fn test_deduplication_preserves_different_urls() {
        // GIVEN: Multiple different URLs (no duplicates)
        let task = TaskBuilder::new()
            .with_description("Visit https://site1.com and https://site2.com")
            .with_comment("user", "Also check https://site3.com and https://site4.com")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: All URLs should be preserved
        assert_eq!(urls.len(), 4, "All different URLs should be preserved");
        
        let found_urls: std::collections::HashSet<String> = urls
            .into_iter()
            .map(|u| u.url)
            .collect();
            
        assert!(found_urls.contains("https://site1.com"));
        assert!(found_urls.contains("https://site2.com"));
        assert!(found_urls.contains("https://site3.com"));
        assert!(found_urls.contains("https://site4.com"));
    }

    #[test]
    fn test_no_urls_returns_empty() {
        // GIVEN: Task with no URLs
        let task = TaskBuilder::new()
            .with_description("This task has no links")
            .with_comment("user", "Just plain text here")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should return empty vector
        assert_eq!(urls.len(), 0, "Task with no URLs should return empty vector");
    }

    #[test]
    fn test_deduplication_with_no_comments() {
        // GIVEN: Task with URLs only in description, no comments
        let task = TaskBuilder::new()
            .with_description("Visit https://example.com and https://example.com again")
            .build();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should deduplicate within description
        assert_eq!(urls.len(), 1, "Duplicate URLs in description should be deduplicated");
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Description");
    }

    #[test]
    fn test_deduplication_with_empty_comments() {
        // GIVEN: Task with empty comments list
        let task = TaskBuilder::new()
            .with_description("Visit https://example.com")
            .build();
        
        // Manually set empty comments (different from None)
        let mut task = task;
        task.comments = Some(vec![]);

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should still work correctly
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Description");
    }

    // Test using the pre-built scenarios from TestScenarios
    #[test]
    fn test_scenarios_duplicate_url_task() {
        // GIVEN: Pre-built duplicate URL scenario
        let task = TestScenarios::duplicate_url_task();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should handle deduplication correctly
        assert_eq!(urls.len(), 1, "TestScenarios::duplicate_url_task should have 1 unique URL");
        assert_eq!(urls[0].url, "https://example.com");
        assert_eq!(urls[0].source, "Description", "Description should win in duplicate scenario");
    }

    #[test]
    fn test_scenarios_html_anchor_task() {
        // GIVEN: Pre-built HTML anchor scenario
        let task = TestScenarios::html_anchor_task();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should not have duplicates from HTML
        assert_eq!(urls.len(), 1, "HTML anchor task should not create duplicate URLs");
        assert_eq!(urls[0].url, "https://site.com");
    }

    #[test]
    fn test_scenarios_complex_deduplication_task() {
        // GIVEN: Pre-built complex scenario
        let task = TestScenarios::complex_deduplication_task();

        // WHEN: Extract URLs
        let urls = extract_urls_from_task(&task);

        // THEN: Should handle complex deduplication correctly
        // Complex scenario has: example.com (desc+comments), unique1.com (desc), unique2.com (comment)
        assert_eq!(urls.len(), 3, "Complex deduplication scenario should have 3 unique URLs");
        
        let url_map: std::collections::HashMap<String, String> = urls
            .into_iter()
            .map(|u| (u.url, u.source))
            .collect();
            
        assert!(url_map.contains_key("https://example.com"));
        assert!(url_map.contains_key("https://unique1.com"));
        assert!(url_map.contains_key("https://unique2.com"));
    }
}

// Integration test with actual task structure to ensure our test builders work correctly
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_task_builder_creates_valid_structure() {
        // Verify our TaskBuilder creates properly structured tasks
        let task = TaskBuilder::new()
            .with_id(42)
            .with_title("Integration Test")
            .with_description("Visit https://test.com")
            .with_comment("author", "Comment with https://comment.com")
            .build();

        // Verify basic structure
        assert_eq!(task.id, 42);
        assert_eq!(task.title, "Integration Test");
        assert!(task.description.is_some());
        assert!(task.comments.is_some());
        
        let comments = task.comments.as_ref().unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].comment.as_ref().unwrap(), "Comment with https://comment.com");
        assert_eq!(comments[0].author.as_ref().unwrap().username, "author");
        
        // Verify URL extraction works with our built task
        let urls = extract_urls_from_task(&task);
        assert_eq!(urls.len(), 2); // One from description, one from comment
    }
}