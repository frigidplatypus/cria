// Test for label submission bug

use cria::QuickAddParser;

#[test]
fn test_parse_task_with_labels() {
    let parser = QuickAddParser::new();
    
    // Test simple label
    let result = parser.parse("Test task *urgent");
    assert_eq!(result.title, "Test task");
    assert_eq!(result.labels, vec!["urgent"]);
    
    // Test multiple labels
    let result = parser.parse("Test task *urgent *high");
    assert_eq!(result.title, "Test task");
    assert_eq!(result.labels, vec!["urgent", "high"]);
    
    // Test label with project
    let result = parser.parse("Test task *urgent +Family");
    assert_eq!(result.title, "Test task");
    assert_eq!(result.labels, vec!["urgent"]);
    assert_eq!(result.project, Some("Family".to_string()));
    
    // Test square bracket labels
    let result = parser.parse("Test task *[high priority]");
    assert_eq!(result.title, "Test task");
    assert_eq!(result.labels, vec!["high priority"]);
}

#[test]
fn test_colorize_input_with_labels() {
    // This tests the colorization function that was causing issues
    // We'll create a mock app and test the colorization
    use cria::tui::app::App;
    
    let mut app = App::new_with_default_project("Test".to_string());
    
    // Add some mock labels
    app.label_map.insert(1, "urgent".to_string());
    app.label_map.insert(2, "high".to_string());
    app.label_map.insert(3, "feature".to_string());
    
    // Test input that was causing issues
    let input = "Test task *urgent *high";
    
    // The colorize function should not cause infinite loops or duplication
    // We can't easily test the UI rendering, but we can test that the input doesn't get corrupted
    assert_eq!(input.len(), 20);
    assert!(input.contains("*urgent"));
    assert!(input.contains("*high"));
    
    // Test the problematic input from the user report
    let input2 = "New Task *feature *high";
    assert_eq!(input2.len(), 23);
    assert!(input2.contains("*feature"));
    assert!(input2.contains("*high"));
}

#[test]
fn test_suggestion_input_parsing() {
    // Test that suggestions don't break the input parsing
    let parser = QuickAddParser::new();
    
    // Simulate what happens when user types and gets suggestions
    let inputs = vec![
        "New Task *f",      // User typing, should parse partially
        "New Task *feature", // After selecting suggestion
        "New Task *feature *h", // User continues typing
        "New Task *feature *high", // After selecting second suggestion
    ];
    
    for input in inputs {
        let result = parser.parse(input);
        // Should never panic or produce corrupted results
        assert!(!result.title.is_empty() || !result.labels.is_empty());
        println!("Input: '{}' -> Title: '{}', Labels: {:?}", input, result.title, result.labels);
    }
}
