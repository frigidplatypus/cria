// Tests for the new column layout and per-layout sorting functionality

use cria::config::{CriaConfig, TaskColumn, TableColumn, ColumnLayout, ColumnSort, SortDirection};
use cria::tui::app::App;
use cria::vikunja::models::Task;
use std::collections::HashMap;

#[test]
fn test_column_layout_creation() {
    let layout = ColumnLayout {
        name: "test-layout".to_string(),
        description: Some("Test layout".to_string()),
        columns: vec![
            TableColumn {
                name: "Task".to_string(),
                column_type: TaskColumn::Title,
                width_percentage: None,
                enabled: true,
                min_width: Some(20),
                max_width: None,
                wrap_text: Some(true),
                sort: Some(ColumnSort {
                    order: 1,
                    direction: SortDirection::Asc,
                }),
            },
            TableColumn {
                name: "Priority".to_string(),
                column_type: TaskColumn::Priority,
                width_percentage: None,
                enabled: true,
                min_width: Some(8),
                max_width: Some(10),
                wrap_text: Some(false),
                sort: Some(ColumnSort {
                    order: 2,
                    direction: SortDirection::Desc,
                }),
            },
        ],
    };
    
    assert_eq!(layout.name, "test-layout");
    assert_eq!(layout.description, Some("Test layout".to_string()));
    assert_eq!(layout.columns.len(), 2);
    
    // Check first column
    assert_eq!(layout.columns[0].name, "Task");
    assert!(matches!(layout.columns[0].column_type, TaskColumn::Title));
    assert!(layout.columns[0].sort.is_some());
    assert_eq!(layout.columns[0].sort.as_ref().unwrap().order, 1);
    assert!(matches!(layout.columns[0].sort.as_ref().unwrap().direction, SortDirection::Asc));
    
    // Check second column
    assert_eq!(layout.columns[1].name, "Priority");
    assert!(matches!(layout.columns[1].column_type, TaskColumn::Priority));
    assert!(layout.columns[1].sort.is_some());
    assert_eq!(layout.columns[1].sort.as_ref().unwrap().order, 2);
    assert!(matches!(layout.columns[1].sort.as_ref().unwrap().direction, SortDirection::Desc));
}

#[test]
fn test_config_column_layouts() {
    let mut config = CriaConfig::default();
    
    // Test default layouts
    let default_layouts = ColumnLayout::default_layouts();
    assert!(!default_layouts.is_empty());
    assert!(default_layouts.iter().any(|l| l.name == "default"));
    assert!(default_layouts.iter().any(|l| l.name == "minimal"));
    
    // Test custom layouts
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "custom".to_string(),
            description: Some("Custom layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Task".to_string(),
                    column_type: TaskColumn::Title,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(30),
                    max_width: None,
                    wrap_text: Some(true),
                    sort: Some(ColumnSort {
                        order: 1,
                        direction: SortDirection::Asc,
                    }),
                },
            ],
        },
    ]);
    
    let layouts = config.get_column_layouts();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].name, "custom");
    assert_eq!(layouts[0].columns.len(), 1);
}

#[test]
fn test_layout_navigation() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "first".to_string(),
            description: Some("First layout".to_string()),
            columns: vec![],
        },
        ColumnLayout {
            name: "second".to_string(),
            description: Some("Second layout".to_string()),
            columns: vec![],
        },
        ColumnLayout {
            name: "third".to_string(),
            description: Some("Third layout".to_string()),
            columns: vec![],
        },
    ]);
    
    config.active_layout = Some("first".to_string());
    
    // Test next_layout cycling
    assert_eq!(config.next_layout("first"), "second");
    assert_eq!(config.next_layout("second"), "third");
    assert_eq!(config.next_layout("third"), "first"); // Should wrap around
    
    // Test previous_layout cycling
    assert_eq!(config.previous_layout("first"), "third"); // Should wrap around
    assert_eq!(config.previous_layout("second"), "first");
    assert_eq!(config.previous_layout("third"), "second");
    
    // Test with non-existent layout (should return first available)
    assert_eq!(config.next_layout("nonexistent"), "first");
    assert_eq!(config.previous_layout("nonexistent"), "first");
}

#[test]
fn test_layout_lookup() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "test".to_string(),
            description: Some("Test layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Task".to_string(),
                    column_type: TaskColumn::Title,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(20),
                    max_width: None,
                    wrap_text: Some(true),
                    sort: None,
                },
            ],
        },
    ]);
    
    // Test successful lookup
    let layout = config.get_layout("test");
    assert!(layout.is_some());
    assert_eq!(layout.unwrap().name, "test");
    
    // Test failed lookup
    let layout = config.get_layout("nonexistent");
    assert!(layout.is_none());
}

#[test]
fn test_app_layout_switching() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "first".to_string(),
            description: Some("First layout".to_string()),
            columns: vec![],
        },
        ColumnLayout {
            name: "second".to_string(),
            description: Some("Second layout".to_string()),
            columns: vec![],
        },
    ]);
    config.active_layout = Some("first".to_string());
    
    let mut app = App::new_with_config(config, "Test".to_string());
    
    // Check initial layout
    assert_eq!(app.current_layout_name, "first");
    
    // Test switching to next layout
    app.switch_to_next_layout();
    assert_eq!(app.current_layout_name, "second");
    
    // Test switching to previous layout
    app.switch_to_previous_layout();
    assert_eq!(app.current_layout_name, "first");
    
    // Test layout info retrieval
    let (name, description) = app.get_current_layout_info();
    assert_eq!(name, "first");
    assert_eq!(description, Some("First layout".to_string()));
}

#[test]
fn test_layout_notification_system() {
    let config = CriaConfig::default();
    let mut app = App::new_with_config(config, "Test".to_string());
    
    // Test showing notification
    app.show_layout_notification("Test notification".to_string());
    assert!(app.layout_notification.is_some());
    assert_eq!(app.layout_notification.as_ref().unwrap(), "Test notification");
    assert!(app.layout_notification_start.is_some());
    
    // Test getting notification (should be visible immediately)
    let notification = app.get_layout_notification();
    assert!(notification.is_some());
    assert_eq!(notification.unwrap(), "Test notification");
    
    // Test notification expiry (simulate time passing)
    std::thread::sleep(std::time::Duration::from_secs(3));
    let notification = app.get_layout_notification();
    assert!(notification.is_none()); // Should be None after 2+ seconds
}

#[test]
fn test_layout_column_retrieval() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "test".to_string(),
            description: Some("Test layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Task".to_string(),
                    column_type: TaskColumn::Title,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(20),
                    max_width: None,
                    wrap_text: Some(true),
                    sort: None,
                },
                TableColumn {
                    name: "Priority".to_string(),
                    column_type: TaskColumn::Priority,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(8),
                    max_width: Some(10),
                    wrap_text: Some(false),
                    sort: None,
                },
            ],
        },
    ]);
    config.active_layout = Some("test".to_string());
    
    let app = App::new_with_config(config, "Test".to_string());
    
    // Test getting columns for current layout
    let columns = app.get_current_layout_columns();
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name, "Task");
    assert_eq!(columns[1].name, "Priority");
}

#[test]
fn test_per_layout_sorting_extraction() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "sorted".to_string(),
            description: Some("Sorted layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Priority".to_string(),
                    column_type: TaskColumn::Priority,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(8),
                    max_width: Some(10),
                    wrap_text: Some(false),
                    sort: Some(ColumnSort {
                        order: 1,
                        direction: SortDirection::Desc,
                    }),
                },
                TableColumn {
                    name: "Task".to_string(),
                    column_type: TaskColumn::Title,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(20),
                    max_width: None,
                    wrap_text: Some(true),
                    sort: Some(ColumnSort {
                        order: 2,
                        direction: SortDirection::Asc,
                    }),
                },
                TableColumn {
                    name: "Due".to_string(),
                    column_type: TaskColumn::DueDate,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(10),
                    max_width: Some(12),
                    wrap_text: Some(false),
                    sort: None, // No sort for this column
                },
            ],
        },
    ]);
    config.active_layout = Some("sorted".to_string());
    
    let app = App::new_with_config(config, "Test".to_string());
    
    // Get the layout and extract sort configuration
    let layout = app.config.get_layout("sorted").unwrap();
    let mut sort_columns: Vec<(&TableColumn, &ColumnSort)> = layout
        .columns
        .iter()
        .filter_map(|col| col.sort.as_ref().map(|sort| (col, sort)))
        .collect();
    
    // Sort by order (primary sort = 1, secondary = 2, etc.)
    sort_columns.sort_by_key(|(_, sort)| sort.order);
    
    assert_eq!(sort_columns.len(), 2); // Only 2 columns have sort config
    
    // Check first sort level (Priority, desc)
    assert!(matches!(sort_columns[0].0.column_type, TaskColumn::Priority));
    assert_eq!(sort_columns[0].1.order, 1);
    assert!(matches!(sort_columns[0].1.direction, SortDirection::Desc));
    
    // Check second sort level (Title, asc)
    assert!(matches!(sort_columns[1].0.column_type, TaskColumn::Title));
    assert_eq!(sort_columns[1].1.order, 2);
    assert!(matches!(sort_columns[1].1.direction, SortDirection::Asc));
}

#[test]
fn test_apply_layout_sort_integration() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "test".to_string(),
            description: Some("Test layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Priority".to_string(),
                    column_type: TaskColumn::Priority,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(8),
                    max_width: Some(10),
                    wrap_text: Some(false),
                    sort: Some(ColumnSort {
                        order: 1,
                        direction: SortDirection::Desc,
                    }),
                },
            ],
        },
    ]);
    config.active_layout = Some("test".to_string());
    
    let mut app = App::new_with_config(config, "Test".to_string());
    
    // Add some test tasks with different priorities
    app.tasks = vec![
        Task {
            id: 1,
            title: "Low priority task".to_string(),
            priority: Some(1),
            project_id: 1,
            done: false,
            description: None,
            done_at: None,
            labels: None,
            assignees: None,
            due_date: None,
            start_date: None,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: false,
            position: None,
            index: None,
            identifier: None,
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: None,
            buckets: None,
            attachments: None,
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: None,
            subscription: None,
        },
        Task {
            id: 2,
            title: "High priority task".to_string(),
            priority: Some(5),
            project_id: 1,
            done: false,
            description: None,
            done_at: None,
            labels: None,
            assignees: None,
            due_date: None,
            start_date: None,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: false,
            position: None,
            index: None,
            identifier: None,
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: None,
            buckets: None,
            attachments: None,
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: None,
            subscription: None,
        },
        Task {
            id: 3,
            title: "No priority task".to_string(),
            priority: None,
            project_id: 1,
            done: false,
            description: None,
            done_at: None,
            labels: None,
            assignees: None,
            due_date: None,
            start_date: None,
            end_date: None,
            created: None,
            updated: None,
            created_by: None,
            percent_done: None,
            is_favorite: false,
            position: None,
            index: None,
            identifier: None,
            hex_color: None,
            cover_image_attachment_id: None,
            bucket_id: None,
            buckets: None,
            attachments: None,
            comments: None,
            reactions: None,
            related_tasks: None,
            reminders: None,
            repeat_after: None,
            repeat_mode: None,
            subscription: None,
        },
    ];
    
    // Apply layout sort
    app.apply_layout_sort();
    
    // Check that tasks are sorted by priority (descending), with None values last
    assert_eq!(app.tasks.len(), 3);
    assert_eq!(app.tasks[0].priority, Some(5)); // High priority first
    assert_eq!(app.tasks[1].priority, Some(1)); // Low priority second
    assert_eq!(app.tasks[2].priority, None);    // No priority last
}

#[test]
fn test_manual_sort_overrides_layout_sort() {
    let mut config = CriaConfig::default();
    config.column_layouts = Some(vec![
        ColumnLayout {
            name: "test".to_string(),
            description: Some("Test layout".to_string()),
            columns: vec![
                TableColumn {
                    name: "Priority".to_string(),
                    column_type: TaskColumn::Priority,
                    width_percentage: None,
                    enabled: true,
                    min_width: Some(8),
                    max_width: Some(10),
                    wrap_text: Some(false),
                    sort: Some(ColumnSort {
                        order: 1,
                        direction: SortDirection::Desc,
                    }),
                },
            ],
        },
    ]);
    config.active_layout = Some("test".to_string());
    
    let mut app = App::new_with_config(config, "Test".to_string());
    
    // Initially, no manual sort is applied
    assert!(app.current_sort.is_none());
    
    // Apply a manual sort
    app.apply_sort(cria::tui::app::SortOrder::TitleAZ);
    
    // Check that manual sort is now active
    assert!(app.current_sort.is_some());
    assert!(matches!(app.current_sort.as_ref().unwrap(), cria::tui::app::SortOrder::TitleAZ));
    
    // Switching layouts should reapply layout sort and clear manual sort
    app.switch_to_next_layout();
    assert!(app.current_sort.is_none()); // Manual sort should be cleared
}

#[test]
fn test_sort_direction_serialization() {
    // Test that sort directions can be serialized/deserialized properly
    let asc_yaml = serde_yaml::to_string(&SortDirection::Asc).unwrap();
    let desc_yaml = serde_yaml::to_string(&SortDirection::Desc).unwrap();
    
    assert_eq!(asc_yaml.trim(), "asc");
    assert_eq!(desc_yaml.trim(), "desc");
    
    // Test deserialization
    let asc_parsed: SortDirection = serde_yaml::from_str("asc").unwrap();
    let desc_parsed: SortDirection = serde_yaml::from_str("desc").unwrap();
    
    assert!(matches!(asc_parsed, SortDirection::Asc));
    assert!(matches!(desc_parsed, SortDirection::Desc));
}

#[test]
fn test_column_sort_serialization() {
    let sort = ColumnSort {
        order: 1,
        direction: SortDirection::Desc,
    };
    
    let yaml = serde_yaml::to_string(&sort).unwrap();
    assert!(yaml.contains("order: 1"));
    assert!(yaml.contains("direction: desc"));
    
    let parsed: ColumnSort = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(parsed.order, 1);
    assert!(matches!(parsed.direction, SortDirection::Desc));
}
