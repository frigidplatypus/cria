use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub key: String,
    pub action: String, // "project", "priority", "label"
    pub target: String, // project name, priority 1-5, or label name
}

impl QuickAction {
    pub fn get_description(&self) -> String {
        match self.action.as_str() {
            "project" => format!("Move to project: {}", self.target),
            "priority" => format!("Set priority to: {}", self.target),
            "label" => format!("Add label: {}", self.target),
            _ => format!("Unknown action: {} -> {}", self.action, self.target),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriaConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub api_key_file: Option<String>,
    pub default_project: Option<String>,
    pub quick_actions: Option<Vec<QuickAction>>,
    pub table_columns: Option<Vec<TableColumn>>,
    pub column_layouts: Option<Vec<ColumnLayout>>,
    pub active_layout: Option<String>,
}

impl Default for CriaConfig {
    fn default() -> Self {
        CriaConfig {
            api_url: "https://vikunja.example.com/api/v1".to_string(),
            api_key: None,
            api_key_file: None,
            default_project: None,
            quick_actions: None,
            table_columns: None,
            column_layouts: None,
            active_layout: None,
        }
    }
}

impl CriaConfig {
    /// Load config from ~/.config/cria/config.yaml (XDG spec)
    #[allow(dead_code)]
    pub fn load() -> Option<Self> {
        Self::load_from_path(None)
    }

    /// Load config from a specific path, or default location if None
    pub fn load_from_path(custom_path: Option<&str>) -> Option<Self> {
        let config_path = if let Some(custom_path) = custom_path {
            PathBuf::from(custom_path)
        } else {
            // Use default XDG location
            match std::env::var("XDG_CONFIG_HOME") {
                Ok(val) => PathBuf::from(val).join("cria/config.yaml"),
                Err(_) => {
                    let mut home = dirs::home_dir()?;
                    home.push(".config/cria/config.yaml");
                    home
                }
            }
        };
        
        let contents = fs::read_to_string(&config_path).ok()?;
        serde_yaml::from_str(&contents).ok()
    }

    /// Check if any API key configuration is present
    pub fn has_api_key_config(&self) -> bool {
        (self.api_key.is_some() && !self.api_key.as_ref().unwrap().trim().is_empty()) ||
        (self.api_key_file.is_some() && !self.api_key_file.as_ref().unwrap().trim().is_empty())
    }

    /// Get the API key, preferring api_key over api_key_file
    pub fn get_api_key(&self) -> Result<String, String> {
        // First try direct api_key
        if let Some(ref key) = self.api_key {
            if !key.trim().is_empty() {
                return Ok(key.clone());
            }
        }

        // Fall back to api_key_file
        if let Some(ref key_file) = self.api_key_file {
            match self.read_api_key_from_file(key_file) {
                Ok(key) => return Ok(key),
                Err(e) => {
                    return Err(format!(
                        "Failed to read API key from file '{}': {}",
                        key_file, e
                    ))
                }
            }
        }

        Err("No API key found. Please set either 'api_key' or 'api_key_file' in config.yaml".to_string())
    }

    /// Read API key from file
    fn read_api_key_from_file(&self, file_path: &str) -> Result<String, std::io::Error> {
        let path = if file_path.starts_with("~/") {
            // Expand tilde to home directory
            let home = dirs::home_dir().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")
            })?;
            home.join(&file_path[2..])
        } else if file_path.starts_with('/') {
            // Absolute path
            PathBuf::from(file_path)
        } else {
            // Relative path - relative to config directory
            let config_dir = match std::env::var("XDG_CONFIG_HOME") {
                Ok(val) => PathBuf::from(val).join("cria"),
                Err(_) => {
                    let mut home = dirs::home_dir().ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::NotFound, "Could not find home directory")
                    })?;
                    home.push(".config/cria");
                    home
                }
            };
            config_dir.join(file_path)
        };

        let contents = fs::read_to_string(path)?;
        Ok(contents.trim().to_string())
    }

    /// Get quick actions as a HashMap keyed by the key character
    #[allow(dead_code)]
    pub fn get_quick_actions_map(&self) -> HashMap<String, QuickAction> {
        self.quick_actions
            .as_ref()
            .map(|actions| {
                actions
                    .iter()
                    .map(|action| (action.key.clone(), action.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a key has a quick action assigned
    #[allow(dead_code)]
    pub fn has_quick_action(&self, key: &str) -> bool {
        self.quick_actions
            .as_ref()
            .map(|actions| actions.iter().any(|action| action.key == key))
            .unwrap_or(false)
    }

    /// Get a quick action by key
    #[allow(dead_code)]
    pub fn get_quick_action(&self, key: &str) -> Option<&QuickAction> {
        self.quick_actions
            .as_ref()
            .and_then(|actions| actions.iter().find(|action| action.key == key))
    }

    /// Get the configured columns, or default columns if none are configured
    pub fn get_columns(&self) -> Vec<TableColumn> {
        self.table_columns.clone().unwrap_or_else(|| TaskColumn::default_columns())
    }

    /// Get the table columns to display, using layouts if configured or falling back to table_columns
    pub fn get_table_columns(&self) -> Vec<TableColumn> {
        // First check if we have column layouts and an active layout
        if let Some(layouts) = &self.column_layouts {
            let active_layout_name = self.active_layout.as_deref().unwrap_or("default");
            if let Some(layout) = layouts.iter().find(|l| l.name == active_layout_name) {
                return layout.columns.clone();
            }
            // If active layout not found, use first layout
            if let Some(first_layout) = layouts.first() {
                return first_layout.columns.clone();
            }
        }
        
        // Fall back to table_columns if no layouts configured
        self.table_columns.clone().unwrap_or_else(|| TaskColumn::default_columns())
    }

    /// Get all available column layouts
    pub fn get_column_layouts(&self) -> Vec<ColumnLayout> {
        self.column_layouts.clone().unwrap_or_else(|| ColumnLayout::default_layouts())
    }

    /// Get the currently active layout name
    pub fn get_active_layout_name(&self) -> String {
        self.active_layout.clone().unwrap_or_else(|| "default".to_string())
    }

    /// Switch to the next layout in the list
    pub fn next_layout(&self, current_layout: &str) -> String {
        let layouts = self.get_column_layouts();
        
        if let Some(current_index) = layouts.iter().position(|l| l.name == current_layout) {
            let next_index = (current_index + 1) % layouts.len();
            layouts[next_index].name.clone()
        } else {
            layouts.first().map(|l| l.name.clone()).unwrap_or_else(|| "default".to_string())
        }
    }

    /// Switch to the previous layout in the list
    pub fn previous_layout(&self, current_layout: &str) -> String {
        let layouts = self.get_column_layouts();
        
        if let Some(current_index) = layouts.iter().position(|l| l.name == current_layout) {
            let prev_index = if current_index == 0 {
                layouts.len() - 1
            } else {
                current_index - 1
            };
            layouts[prev_index].name.clone()
        } else {
            layouts.first().map(|l| l.name.clone()).unwrap_or_else(|| "default".to_string())
        }
    }

    /// Get layout by name
    pub fn get_layout(&self, name: &str) -> Option<ColumnLayout> {
        self.get_column_layouts().into_iter().find(|l| l.name == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub name: String,
    pub column_type: TaskColumn,
    #[serde(default)]
    pub width_percentage: Option<u16>, // Make width optional
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub min_width: Option<u16>, // Minimum width in characters
    #[serde(default)]
    pub max_width: Option<u16>, // Maximum width in characters
    #[serde(default)]
    pub wrap_text: Option<bool>, // Whether to wrap text in this column
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskColumn {
    Title,
    Project,
    Labels,
    DueDate,
    StartDate,
    Priority,
    Status,
    Assignees,
    Created,
    Updated,
}

impl TaskColumn {
    pub fn default_columns() -> Vec<TableColumn> {
        vec![
            TableColumn {
                name: "Title".to_string(),
                column_type: TaskColumn::Title,
                width_percentage: None, // Auto-calculate
                enabled: true,
                min_width: Some(20),
                max_width: None, // No max for title
                wrap_text: Some(true), // Wrap task titles
            },
            TableColumn {
                name: "Project".to_string(),
                column_type: TaskColumn::Project,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(20),
                wrap_text: Some(false),
            },
            TableColumn {
                name: "Due Date".to_string(),
                column_type: TaskColumn::DueDate,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(12),
                wrap_text: Some(false),
            },
            TableColumn {
                name: "Start Date".to_string(),
                column_type: TaskColumn::StartDate,
                width_percentage: None,
                enabled: true,
                min_width: Some(10),
                max_width: Some(12),
                wrap_text: Some(false),
            },
            TableColumn {
                name: "Labels".to_string(),
                column_type: TaskColumn::Labels,
                width_percentage: None,
                enabled: true,
                min_width: Some(8),
                max_width: Some(25),
                wrap_text: Some(true),
            },
        ]
    }

    pub fn get_display_name(&self) -> &'static str {
        match self {
            TaskColumn::Title => "Title",
            TaskColumn::Project => "Project",
            TaskColumn::Labels => "Labels",
            TaskColumn::DueDate => "Due Date",
            TaskColumn::StartDate => "Start Date",
            TaskColumn::Priority => "Priority",
            TaskColumn::Status => "Status",
            TaskColumn::Assignees => "Assignees",
            TaskColumn::Created => "Created",
            TaskColumn::Updated => "Updated",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnLayout {
    pub name: String,
    pub description: Option<String>,
    pub columns: Vec<TableColumn>,
}

impl ColumnLayout {
    pub fn default_layouts() -> Vec<ColumnLayout> {
        vec![
            ColumnLayout {
                name: "default".to_string(),
                description: Some("Standard task view with all essential columns".to_string()),
                columns: TaskColumn::default_columns(),
            },
            ColumnLayout {
                name: "minimal".to_string(),
                description: Some("Clean, minimal view with just task and due date".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(30),
                        max_width: None,
                        wrap_text: Some(true),
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(15),
                        wrap_text: Some(false),
                    },
                ],
            },
            ColumnLayout {
                name: "project-focused".to_string(),
                description: Some("Project-centric view for team collaboration".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(12),
                        max_width: Some(20),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(25),
                        max_width: None,
                        wrap_text: Some(true),
                    },
                    TableColumn {
                        name: "Priority".to_string(),
                        column_type: TaskColumn::Priority,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(8),
                        max_width: Some(10),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Labels".to_string(),
                        column_type: TaskColumn::Labels,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(8),
                        max_width: Some(20),
                        wrap_text: Some(true),
                    },
                ],
            },
            ColumnLayout {
                name: "time-management".to_string(),
                description: Some("Time-focused view for scheduling and deadlines".to_string()),
                columns: vec![
                    TableColumn {
                        name: "Task".to_string(),
                        column_type: TaskColumn::Title,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(25),
                        max_width: None,
                        wrap_text: Some(true),
                    },
                    TableColumn {
                        name: "Start".to_string(),
                        column_type: TaskColumn::StartDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Due".to_string(),
                        column_type: TaskColumn::DueDate,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Created".to_string(),
                        column_type: TaskColumn::Created,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(12),
                        wrap_text: Some(false),
                    },
                    TableColumn {
                        name: "Project".to_string(),
                        column_type: TaskColumn::Project,
                        width_percentage: None,
                        enabled: true,
                        min_width: Some(10),
                        max_width: Some(15),
                        wrap_text: Some(false),
                    },
                ],
            },
        ]
    }
}
