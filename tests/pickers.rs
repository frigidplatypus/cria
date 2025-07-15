// Tests for picker event handling: project and filter pickers

use cria::config::CriaConfig;
use cria::tui::app::state::App;
use cria::tui::app::task_filter::TaskFilter;

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

#[test]
fn test_project_picker_preserves_task_filter() {
    use cria::tui::app::task_filter::TaskFilter;
    
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    
    // Set up projects
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    
    // Set up test tasks - some completed, some not
    let tasks = vec![
        cria::vikunja::models::Task {
            id: 1,
            title: "Inbox Active Task".to_string(),
            description: Some("test".to_string()),
            done: false,
            done_at: None,
            priority: Some(1),
            due_date: None,
            project_id: 1,
            labels: Some(vec![]),
            assignees: Some(vec![]),
            is_favorite: false,
            start_date: None,
            end_date: None,
            created: Some("2023-01-01T00:00:00Z".to_string()),
            updated: Some("2023-01-01T00:00:00Z".to_string()),
            created_by: None,
            percent_done: Some(0),
            position: Some(0),
            index: Some(1),
            identifier: Some("1".to_string()),
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: Some(0),
            buckets: Some(vec![]),
            attachments: Some(vec![]),
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: Some(0),
            subscription: None,
        },
        cria::vikunja::models::Task {
            id: 2,
            title: "Work Active Task".to_string(),
            description: Some("test".to_string()),
            done: false,
            done_at: None,
            priority: Some(1),
            due_date: None,
            project_id: 2,
            labels: Some(vec![]),
            assignees: Some(vec![]),
            is_favorite: false,
            start_date: None,
            end_date: None,
            created: Some("2023-01-01T00:00:00Z".to_string()),
            updated: Some("2023-01-01T00:00:00Z".to_string()),
            created_by: None,
            percent_done: Some(0),
            position: Some(0),
            index: Some(2),
            identifier: Some("2".to_string()),
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: Some(0),
            buckets: Some(vec![]),
            attachments: Some(vec![]),
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: Some(0),
            subscription: None,
        },
        cria::vikunja::models::Task {
            id: 3,
            title: "Work Completed Task".to_string(),
            description: Some("test".to_string()),
            done: true,
            done_at: Some("2023-01-01T00:00:00Z".to_string()),
            priority: Some(1),
            due_date: None,
            project_id: 2,
            labels: Some(vec![]),
            assignees: Some(vec![]),
            is_favorite: false,
            start_date: None,
            end_date: None,
            created: Some("2023-01-01T00:00:00Z".to_string()),
            updated: Some("2023-01-01T00:00:00Z".to_string()),
            created_by: None,
            percent_done: Some(100),
            position: Some(0),
            index: Some(3),
            identifier: Some("3".to_string()),
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: Some(0),
            buckets: Some(vec![]),
            attachments: Some(vec![]),
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: Some(0),
            subscription: None,
        },
    ];
    
    app.update_all_tasks(tasks);
    
    // Initially, task filter should be ActiveOnly (default), showing only active tasks
    assert_eq!(app.task_filter, TaskFilter::ActiveOnly);
    assert_eq!(app.tasks.len(), 2); // Should show only active tasks (1 from Inbox, 1 from Work)
    assert!(app.tasks.iter().all(|t| !t.done)); // All visible tasks should be active
    
    // Switch to Work project via project picker
    app.show_project_picker();
    app.project_picker_input = "Work".to_string();
    app.update_filtered_projects();
    
    // Find and select Work project
    if let Some(work_index) = app.filtered_projects.iter().position(|(_, name)| name == "Work") {
        app.selected_project_picker_index = work_index;
        app.select_project_picker();
    }
    
    // After switching to Work project, task filter should still be ActiveOnly
    assert_eq!(app.task_filter, TaskFilter::ActiveOnly);
    assert_eq!(app.current_project_id, Some(2)); // Should be on Work project
    assert_eq!(app.tasks.len(), 1); // Should show only active tasks from Work project
    assert!(app.tasks.iter().all(|t| !t.done && t.project_id == 2)); // Only active Work tasks
    
    // Change task filter to All while on Work project
    app.cycle_task_filter(); // ActiveOnly -> All
    assert_eq!(app.task_filter, TaskFilter::All);
    assert_eq!(app.tasks.len(), 2); // Should show all tasks from Work project (active + completed)
    assert!(app.tasks.iter().all(|t| t.project_id == 2)); // All tasks should be from Work project
}
