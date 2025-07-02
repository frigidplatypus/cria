#[cfg(test)]
mod tests {
    use cria::config::CriaConfig;
    use cria::tui::app::App;
    use cria::vikunja_parser::QuickAddParser;

    #[test]
    fn test_input_flow_debug() {
        let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
        
        // Simulate typing "Test task *urgent"
        let input_text = "Test task *urgent";
        app.quick_add_input = input_text.to_string();
        app.quick_add_cursor_position = input_text.len();
        
        println!("Original input: '{}'", input_text);
        println!("App input: '{}'", app.get_quick_add_input());
        
        // Test the parser directly
        let parser = QuickAddParser::new();
        let parsed = parser.parse(input_text);
        println!("Parsed title: '{}'", parsed.title);
        println!("Parsed labels: {:?}", parsed.labels);
        
        // Test what the app would parse
        let app_parsed = parser.parse(app.get_quick_add_input());
        println!("App parsed title: '{}'", app_parsed.title);
        println!("App parsed labels: {:?}", app_parsed.labels);
        
        assert_eq!(parsed.labels.len(), 1);
        assert_eq!(parsed.labels[0], "urgent");
        assert_eq!(app_parsed.labels.len(), 1);
        assert_eq!(app_parsed.labels[0], "urgent");
    }
}
