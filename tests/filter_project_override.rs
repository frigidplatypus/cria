use cria::tui::app::state::App;
use cria::config::CriaConfig;

#[test]
fn test_project_override_extraction() {
    let config = CriaConfig::default();
    let mut app = App::new_with_config(config, "Default Project".to_string());
    
    // Test setting filters with descriptions
    let filters = vec![
        (1, "Personal Tasks".to_string(), Some("Personal task filter".to_string())),
        (2, "Work Tasks".to_string(), Some("cria_project: WorkProject".to_string())),
        (3, "Urgent Tasks".to_string(), Some("Filter for urgent tasks".to_string())),
        (4, "Custom Filter".to_string(), Some("Custom filter with cria_project: CustomProject and other stuff".to_string())),
    ];
    
    app.set_filters(filters);
    
    // Test extraction of project override
    assert_eq!(app.extract_project_override(1), None);
    assert_eq!(app.extract_project_override(2), Some("WorkProject".to_string()));
    assert_eq!(app.extract_project_override(3), None);
    assert_eq!(app.extract_project_override(4), Some("CustomProject".to_string()));
    
    // Test non-existent filter
    assert_eq!(app.extract_project_override(999), None);
    
    // Test that default project is correct initially
    assert_eq!(app.get_active_default_project(), "Default Project");
    
    // Test applying filter with project override
    app.apply_filter_with_override(2);
    assert_eq!(app.current_filter_id, Some(2));
    assert_eq!(app.active_project_override, Some("WorkProject".to_string()));
    assert_eq!(app.get_active_default_project(), "WorkProject");
    
    // Test clearing filter resets project override
    app.clear_filter();
    assert_eq!(app.current_filter_id, None);
    assert_eq!(app.active_project_override, None);
    assert_eq!(app.get_active_default_project(), "Default Project");
}

#[test]
fn test_filter_project_override_edge_cases() {
    let config = CriaConfig::default();
    let mut app = App::new_with_config(config, "DefaultProject".to_string());
    
    // Test edge cases for project override extraction
    let filters = vec![
        (1, "Test 1".to_string(), Some("cria_project:".to_string())), // Empty project name
        (2, "Test 2".to_string(), Some("cria_project: ".to_string())), // Space but empty
        (3, "Test 3".to_string(), Some("cria_project:SingleWord".to_string())), // No space after colon
        (4, "Test 4".to_string(), Some("cria_project: MultiWord Project".to_string())), // Multiple words
        (5, "Test 5".to_string(), Some("some text cria_project: MidText more text".to_string())), // Middle of text
    ];
    
    app.set_filters(filters);
    
    // Test edge cases
    assert_eq!(app.extract_project_override(1), None); // Empty project name
    assert_eq!(app.extract_project_override(2), None); // Empty project name with space
    assert_eq!(app.extract_project_override(3), Some("SingleWord".to_string())); // No space after colon
    assert_eq!(app.extract_project_override(4), Some("MultiWord".to_string())); // Only first word
    assert_eq!(app.extract_project_override(5), Some("MidText".to_string())); // From middle of text
}

#[test]
fn test_filter_picker_clear_option() {
    let config = CriaConfig::default();
    let mut app = App::new_with_config(config, "Default Project".to_string());
    
    // Set up filters
    let filters = vec![
        (1, "Work Filter".to_string(), Some("cria_project: WorkProject".to_string())),
        (2, "Personal Filter".to_string(), Some("Personal tasks only".to_string())),
    ];
    app.set_filters(filters);
    
    // Apply a filter first
    app.apply_filter_with_override(1);
    assert_eq!(app.current_filter_id, Some(1));
    assert_eq!(app.active_project_override, Some("WorkProject".to_string()));
    
    // Open filter picker and check that "Clear Filter" option is added
    app.show_filter_picker();
    app.update_filtered_filters();
    
    // Should have 3 items: "Clear Filter" + 2 original filters
    assert_eq!(app.filtered_filters.len(), 3);
    assert_eq!(app.filtered_filters[0], (-1, "Clear Filter".to_string()));
    assert_eq!(app.filtered_filters[1], (1, "Work Filter".to_string()));
    assert_eq!(app.filtered_filters[2], (2, "Personal Filter".to_string()));
    
    // Test that selecting "Clear Filter" clears the current filter
    app.selected_filter_picker_index = 0; // Select "Clear Filter"
    // Simulate what happens when "Clear Filter" is selected
    app.clear_filter();
    app.apply_task_filter();
    
    assert_eq!(app.current_filter_id, None);
    assert_eq!(app.active_project_override, None);
    assert_eq!(app.get_active_default_project(), "Default Project");
    
    // After clearing, filter picker should not show "Clear Filter" option
    app.update_filtered_filters();
    assert_eq!(app.filtered_filters.len(), 2);
    assert_eq!(app.filtered_filters[0], (1, "Work Filter".to_string()));
    assert_eq!(app.filtered_filters[1], (2, "Personal Filter".to_string()));
}

#[test]
fn test_default_filter_functionality() {
    let config = CriaConfig::default();
    let mut app = App::new_with_config(config, "Default Project".to_string());
    
    // Set up filters
    let filters = vec![
        (1, "Work Filter".to_string(), Some("cria_project: WorkProject".to_string())),
        (2, "Daily Tasks".to_string(), Some("Tasks for daily review".to_string())),
        (3, "Weekly Review".to_string(), Some("Weekly planning filter".to_string())),
    ];
    app.set_filters(filters);
    
    // Test finding filter by name (case insensitive)
    assert_eq!(app.find_filter_by_name("Work Filter"), Some(1));
    assert_eq!(app.find_filter_by_name("work filter"), Some(1));
    assert_eq!(app.find_filter_by_name("WORK FILTER"), Some(1));
    assert_eq!(app.find_filter_by_name("Daily Tasks"), Some(2));
    assert_eq!(app.find_filter_by_name("Nonexistent Filter"), None);
    
    // Test that initially no filter is applied
    assert_eq!(app.current_filter_id, None);
    assert_eq!(app.active_project_override, None);
}

#[test]
fn test_config_with_default_filter() {
    let config = CriaConfig {
        api_url: "https://example.com/api/v1".to_string(),
        api_key: Some("test-key".to_string()),
        api_key_file: None,
        default_project: Some("Inbox".to_string()),
        default_filter: Some("Daily Tasks".to_string()),
        quick_actions: None,
        table_columns: None,
        column_layouts: None,
        active_layout: None,
        refresh_interval_seconds: Some(300),
        auto_refresh: Some(true),
    };
    
    assert_eq!(config.default_filter, Some("Daily Tasks".to_string()));
    
    // Test default config has no default filter
    let default_config = CriaConfig::default();
    assert_eq!(default_config.default_filter, None);
}
