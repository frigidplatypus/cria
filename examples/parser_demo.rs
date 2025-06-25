// examples/parser_demo.rs
use cria::vikunja_parser::QuickAddParser;

fn main() {
    let parser = QuickAddParser::new();
    
    println!("=== Enhanced Quick Add Magic Parser Demo ===\n");
    
    let test_cases = vec![
        "Buy groceries *shopping @john +personal tomorrow !2",
        r#"Review proposal *urgent *"high priority" @jane @"john doe" +"Client Work" next monday at 10am !4 every week"#,
        "Call mom next friday at 2:30pm",
        "Pay rent 15th",
        "Follow up in 3 days",
        "Team meeting this weekend *important",
        "Submit report Feb 17th at 5pm",
        "Clean garage *weekend +home every 2 weeks",
        "Doctor appointment tomorrow at 9:00am *health",
        "Project deadline end of month !5 @team +work",
    ];
    
    for (i, input) in test_cases.iter().enumerate() {
        println!("Test case {}: \"{}\"", i + 1, input);
        let task = parser.parse(input);
        
        println!("  Title: '{}'", task.title);
        if !task.labels.is_empty() {
            println!("  Labels: {}", task.labels.join(", "));
        }
        if !task.assignees.is_empty() {
            println!("  Assignees: {}", task.assignees.join(", "));
        }
        if let Some(project) = &task.project {
            println!("  Project: {}", project);
        }
        if let Some(priority) = task.priority {
            println!("  Priority: {}", priority);
        }
        if let Some(due_date) = task.due_date {
            println!("  Due date: {}", due_date.format("%Y-%m-%d %H:%M UTC"));
        }
        if let Some(repeat) = &task.repeat_interval {
            println!("  Repeats: every {} {}", repeat.amount, repeat.interval_type);
        }
        println!();
    }
    
    println!("=== Features Demonstrated ===");
    println!("✓ Labels with *label or *\"label with spaces\"");
    println!("✓ Assignees with @user or @\"user name\"");
    println!("✓ Projects with +project or +\"project with spaces\"");
    println!("✓ Priority with !1-5");
    println!("✓ Natural language dates (today, tomorrow, next monday, etc.)");
    println!("✓ Time parsing (at 2:30pm, at 10am)");
    println!("✓ Specific dates (15th, Feb 17th)");
    println!("✓ Duration dates (in 3 days)");
    println!("✓ Repeat intervals (every 2 weeks)");
    println!("✓ Intelligent title cleaning");
}
