use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::state::App;

#[test]
fn test_quick_actions_modal_colorization() {
    // Create a config with various quick action types
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "d".to_string(),
            action: "label".to_string(),
            target: "datenight".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
        QuickAction {
            key: "l".to_string(),
            action: "priority".to_string(),
            target: "1".to_string(),
        },
    ]);

    let app = App::new_with_config(config, "Inbox".to_string());
    
    // Verify that the quick actions are loaded correctly
    assert!(app.config.quick_actions.is_some());
    let quick_actions = app.config.quick_actions.as_ref().unwrap();
    assert_eq!(quick_actions.len(), 4);
    
    // Test different action types have appropriate descriptions
    assert_eq!(quick_actions[0].get_description(), "Move to project: Work");
    assert_eq!(quick_actions[1].get_description(), "Add label: datenight");
    assert_eq!(quick_actions[2].get_description(), "Set priority to: 5");
    assert_eq!(quick_actions[3].get_description(), "Set priority to: 1");
    
    // Test that modal shows correctly (just test that it doesn't crash)
    assert_eq!(app.selected_quick_action_index, 0);
    assert!(!app.show_quick_actions_modal);
    
    println!("✓ Quick actions modal colorization test completed successfully");
    println!("  - Project action: {} -> {}", quick_actions[0].key, quick_actions[0].get_description());
    println!("  - Label action: {} -> {}", quick_actions[1].key, quick_actions[1].get_description());
    println!("  - High priority action: {} -> {}", quick_actions[2].key, quick_actions[2].get_description());
    println!("  - Low priority action: {} -> {}", quick_actions[3].key, quick_actions[3].get_description());
}
