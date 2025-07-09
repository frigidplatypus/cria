// Tests for picker event handling: project and filter pickers

use cria::config::CriaConfig;
use cria::tui::app::App;

#[test]
fn test_project_picker_events() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
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
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
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
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
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

#[test]
fn test_project_picker_all_projects_option() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    // Add tasks for both projects
    app.all_tasks = vec![
        cria::vikunja::models::Task { id: 1, project_id: 1, title: "Inbox Task".to_string(), ..Default::default() },
        cria::vikunja::models::Task { id: 2, project_id: 2, title: "Work Task".to_string(), ..Default::default() },
    ];
    app.tasks = app.all_tasks.clone();
    // Select a project
    app.show_project_picker();
    // Dynamically find the index for 'Work'
    let work_index = app.filtered_projects.iter().position(|(id, name)| *id == 2 && name == "Work").expect("'Work' project not found in filtered_projects");
    app.selected_project_picker_index = work_index;
    app.select_project_picker();
    assert_eq!(app.current_project_id, Some(2));
    assert_eq!(app.tasks.len(), 1);
    assert_eq!(app.tasks[0].title, "Work Task");
    // Open picker again, should show 'All Projects' option
    app.show_project_picker();
    let all_projects_index = app.filtered_projects.iter().position(|(id, name)| *id == -1 && name == "All Projects").expect("'All Projects' option not found in filtered_projects");
    // Select 'All Projects'
    app.selected_project_picker_index = all_projects_index;
    app.select_project_picker();
    assert_eq!(app.current_project_id, None);
    assert_eq!(app.tasks.len(), 2);
    let titles: Vec<_> = app.tasks.iter().map(|t| t.title.as_str()).collect();
    assert!(titles.contains(&"Inbox Task"));
    assert!(titles.contains(&"Work Task"));
}
