// Tests for the customizable table column functionality

use cria::config::{CriaConfig, TaskColumn, TableColumn};
use cria::tui::app::state::App;

#[test]
fn test_default_table_columns() {
    let columns = TaskColumn::default_columns();
    
    // Verify we have the expected default columns
    assert_eq!(columns.len(), 5);
    assert!(columns.iter().any(|c| matches!(c.column_type, TaskColumn::Title)));
    assert!(columns.iter().any(|c| matches!(c.column_type, TaskColumn::Project)));
    assert!(columns.iter().any(|c| matches!(c.column_type, TaskColumn::DueDate)));
    assert!(columns.iter().any(|c| matches!(c.column_type, TaskColumn::StartDate)));
    assert!(columns.iter().any(|c| matches!(c.column_type, TaskColumn::Labels)));
    
    // Verify all default columns are enabled
    assert!(columns.iter().all(|c| c.enabled));
    
    // Verify auto-width calculation (no fixed widths)
    assert!(columns.iter().all(|c| c.width_percentage.is_none()));
    assert!(columns.iter().all(|c| c.min_width.is_some()));
}

#[test]
fn test_config_table_columns() {
    let mut config = CriaConfig::default();
    
    // Test default behavior (no table_columns configured)
    let columns = config.get_table_columns();
    assert_eq!(columns.len(), 5);
    
    // Test with custom configuration
    config.table_columns = Some(vec![
        TableColumn {
            name: "Task".to_string(),
            column_type: TaskColumn::Title,
            width_percentage: Some(60),
            enabled: true,
            min_width: Some(20),
            max_width: None,
            wrap_text: Some(true),
            sort: None,
        },
        TableColumn {
            name: "Due".to_string(),
            column_type: TaskColumn::DueDate,
            width_percentage: Some(40),
            enabled: true,
            min_width: Some(10),
            max_width: Some(12),
            wrap_text: Some(false),
            sort: None,
        },
    ]);
    
    let columns = config.get_table_columns();
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name, "Task");
    assert_eq!(columns[1].name, "Due");
}

#[test]
fn test_column_display_names() {
    assert_eq!(TaskColumn::Title.get_display_name(), "Title");
    assert_eq!(TaskColumn::DueDate.get_display_name(), "Due Date");
    assert_eq!(TaskColumn::StartDate.get_display_name(), "Start Date");
    assert_eq!(TaskColumn::Priority.get_display_name(), "Priority");
    assert_eq!(TaskColumn::Status.get_display_name(), "Status");
}

#[test]
fn test_app_uses_config_columns() {
    let mut config = CriaConfig::default();
    config.table_columns = Some(vec![
        TableColumn {
            name: "Custom Title".to_string(),
            column_type: TaskColumn::Title,
            width_percentage: Some(50),
            enabled: true,
            min_width: Some(20),
            max_width: None,
            wrap_text: Some(true),
            sort: None,
        },
        TableColumn {
            name: "Custom Priority".to_string(),
            column_type: TaskColumn::Priority,
            width_percentage: Some(50),
            enabled: true,
            min_width: Some(8),
            max_width: Some(10),
            wrap_text: Some(false),
            sort: None,
        },
    ]);
    
    let app = App::new_with_config(config, "Test".to_string());
    let columns = app.config.get_table_columns();
    
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name, "Custom Title");
    assert_eq!(columns[1].name, "Custom Priority");
}

#[test]
fn test_disabled_columns_filtering() {
    let columns = vec![
        TableColumn {
            name: "Enabled".to_string(),
            column_type: TaskColumn::Title,
            width_percentage: Some(50),
            enabled: true,
            min_width: Some(20),
            max_width: None,
            wrap_text: Some(true),
            sort: None,
        },
        TableColumn {
            name: "Disabled".to_string(),
            column_type: TaskColumn::Priority,
            width_percentage: Some(50),
            enabled: false,
            min_width: Some(8),
            max_width: Some(10),
            wrap_text: Some(false),
            sort: None,
        },
    ];
    
    let enabled: Vec<&TableColumn> = columns.iter().filter(|c| c.enabled).collect();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].name, "Enabled");
}
