// Test the new square bracket syntax parsing
use cria::vikunja_parser::QuickAddParser;

#[test]
fn test_square_bracket_project_parsing() {
    let parser = QuickAddParser::new();
    
    // Test square bracket project syntax
    let parsed = parser.parse("Review code +[Home Maintenance] *urgent");
    assert_eq!(parsed.title, "Review code");
    assert_eq!(parsed.project, Some("Home Maintenance".to_string()));
    assert_eq!(parsed.labels, vec!["urgent"]);
}

#[test]
fn test_square_bracket_label_parsing() {
    let parser = QuickAddParser::new();
    
    // Test square bracket label syntax
    let parsed = parser.parse("Fix bug *[code review] +work");
    assert_eq!(parsed.title, "Fix bug");
    assert_eq!(parsed.labels, vec!["code review"]);
    assert_eq!(parsed.project, Some("work".to_string()));
}

#[test]
fn test_mixed_bracket_styles() {
    let parser = QuickAddParser::new();
    
    // Test mixing different bracket styles
    let parsed = parser.parse("Task with +\"quoted project\" *[bracket label] *'single quoted'");
    assert_eq!(parsed.title, "Task with");
    assert_eq!(parsed.project, Some("quoted project".to_string()));
    assert_eq!(parsed.labels, vec!["bracket label", "single quoted"]);
}

#[test] 
fn test_single_word_vs_multi_word() {
    let parser = QuickAddParser::new();
    
    // Test that single words still work without brackets
    let parsed1 = parser.parse("Simple task +work *urgent");
    assert_eq!(parsed1.project, Some("work".to_string()));
    assert_eq!(parsed1.labels, vec!["urgent"]);
    
    // Test that brackets work for multi-word
    let parsed2 = parser.parse("Complex task +[work project] *[high priority]");
    assert_eq!(parsed2.project, Some("work project".to_string()));
    assert_eq!(parsed2.labels, vec!["high priority"]);
}
