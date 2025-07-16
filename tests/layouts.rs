use cria::config::{CriaConfig, SortDirection};
use cria::tui::app::state::App;
use cria::vikunja::models::Task;
use chrono::{Utc, TimeZone};

fn sample_tasks() -> Vec<Task> {
    vec![
        Task {
            id: 1,
            title: "Alpha".to_string(),
            project_id: 1,
            priority: Some(2),
            due_date: Utc.with_ymd_and_hms(2025, 7, 10, 12, 0, 0).single(),
            ..Default::default()
        },
        Task {
            id: 2,
            title: "Bravo".to_string(),
            project_id: 2,
            priority: Some(3),
            due_date: None,
            ..Default::default()
        },
        Task {
            id: 3,
            title: "Charlie".to_string(),
            project_id: 1,
            priority: None,
            due_date: Utc.with_ymd_and_hms(2025, 7, 5, 12, 0, 0).single(),
            ..Default::default()
        },
        Task {
            id: 4,
            title: "Delta".to_string(),
            project_id: 2,
            priority: Some(1),
            due_date: None,
            ..Default::default()
        },
    ]
}

#[test]
fn test_layout_parsing_and_sort_config() {
    let yaml = std::fs::read_to_string("cargo-test.config.yaml").expect("cargo-test.config.yaml missing");
    let config: CriaConfig = serde_yaml::from_str(&yaml).expect("Failed to parse config");

    // Check that layouts are parsed
    let layouts = config.get_column_layouts();
    assert!(layouts.iter().any(|l| l.name == "default"));
    let default = layouts.iter().find(|l| l.name == "default").unwrap();

    // Check sort config for default layout
    let prio_col = default.columns.iter().find(|c| c.name == "Priority").unwrap();
    assert_eq!(prio_col.sort.as_ref().unwrap().order, 1);
    assert_eq!(prio_col.sort.as_ref().unwrap().direction, SortDirection::Desc);

    let title_col = default.columns.iter().find(|c| c.name == "Task").unwrap();
    assert_eq!(title_col.sort.as_ref().unwrap().order, 2);
    assert_eq!(title_col.sort.as_ref().unwrap().direction, SortDirection::Asc);
}

#[test]
fn test_layout_sorting_logic() {
    let yaml = std::fs::read_to_string("cargo-test.config.yaml").expect("cargo-test.config.yaml missing");
    let config: CriaConfig = serde_yaml::from_str(&yaml).expect("Failed to parse config");
    let mut app = App::new_with_config(config.clone(), "Inbox".to_string());

    // Simulate loading tasks and switching to the default layout
    let tasks = sample_tasks();
    app.tasks = tasks.clone();
    app.all_tasks = tasks.clone();
    app.current_layout_name = "default".to_string();
    app.apply_layout_sort();

    // Should be sorted by Priority (desc), then Title (asc)
    let sorted_titles: Vec<_> = app.tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(sorted_titles, vec!["Bravo", "Alpha", "Delta", "Charlie"]); // 3,2,1,None

    // Switch to minimal layout (Title asc, Due asc)
    app.current_layout_name = "minimal".to_string();
    app.apply_layout_sort();
    let sorted_titles: Vec<_> = app.tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(sorted_titles, vec!["Alpha", "Bravo", "Charlie", "Delta"]);

    // Switch to timeline layout (Start asc, Due asc) - but our sample tasks have no start, so Due asc
    app.current_layout_name = "timeline".to_string();
    app.apply_layout_sort();
    let sorted_titles: Vec<_> = app.tasks.iter().map(|t| t.title.as_str()).collect();
    // Due dates: Charlie (2025-07-05), Alpha (2025-07-10), Bravo/Delta (None, should be last)
    assert_eq!(sorted_titles, vec!["Charlie", "Alpha", "Bravo", "Delta"]);
}

#[test]
fn test_nulls_sort_last() {
    let mut tasks = sample_tasks();
    // Set all priorities to None for one task
    tasks[2].priority = None;
    // Sort by priority asc, nulls last
    tasks.sort_by(|a, b| {
        let cmp = match (a.priority, b.priority) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(a), Some(b)) => a.cmp(&b),
        };
        cmp
    });
    let sorted_titles: Vec<_> = tasks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(sorted_titles.last().unwrap(), &"Charlie");
}
