use cria::vikunja_parser::QuickAddParser;

#[test]
fn test_label_parsing_debug() {
    let mut parser = QuickAddParser::new();
    
    // Test various label inputs
    let test_cases = vec![
        "Test task *urgent",
        "Test task *urgent *high",
        "Task with *feature label",
        "*urgent task",
        "Task *urgent more text",
    ];
    
    for input in test_cases {
        println!("\n--- Testing input: '{}' ---", input);
        let result = parser.parse(input);
        println!("Title: '{}'", result.title);
        println!("Labels: {:?}", result.labels);
        println!("Project: {:?}", result.project);
        println!("Priority: {:?}", result.priority);
        
        // Debug: show what the parser found
        if result.labels.is_empty() {
            println!("❌ No labels found!");
        } else {
            println!("✅ Found {} labels", result.labels.len());
        }
    }
}
