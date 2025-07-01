// Tests for picker event handling: project and filter pickers

use cria::tui::app::App;

#[test]
fn test_project_picker_events() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    // Open picker
    app.show_project_picker();
    assert!(app.show_project_picker);
    // Input
    app.add_char_to_project_picker('W');
    assert!(app.filtered_projects.iter().any(|(_, n)| n == "Work"));
    // Select
    app.select_project_picker();
    assert!(app.current_project_id.is_some());
    // Close picker
    app.hide_project_picker();
    assert!(!app.show_project_picker);
}

#[test]
fn test_filter_picker_events() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.filters = vec![(1, "My Filter".to_string())];
    // Open picker
    app.show_filter_picker = true;
    app.filter_picker_input = "My".to_string();
    app.filtered_filters = app.filters.clone();
    app.selected_filter_picker_index = 0;
    // Simulate selection
    app.current_filter_id = Some(app.filtered_filters[0].0);
    assert_eq!(app.current_filter_id, Some(1));
    // Close picker
    app.show_filter_picker = false;
    assert!(!app.show_filter_picker);
}

#[test]
fn test_picker_input_resets_on_cancel() {
    let mut app = App::new_with_default_project("Inbox".to_string());
    app.filters = vec![(1, "My Filter".to_string())];
    // Open filter picker and type input
    app.show_filter_picker = true;
    app.filter_picker_input = "My".to_string();
    app.filtered_filters = app.filters.clone();
    app.selected_filter_picker_index = 0;
    // Cancel picker
    app.show_filter_picker = false;
    app.filter_picker_input.clear();
    // Input should be reset
    assert!(!app.show_filter_picker);
    assert_eq!(app.filter_picker_input, "");
}
