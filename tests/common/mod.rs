//! Common utilities for integration tests

use cria::config::{CriaConfig, QuickAction};
use cria::tui::app::state::App;
use cria::vikunja::models::Task;
use chrono::{NaiveDate, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn get_env_var(keys: &[&str], default: &str) -> String {
    for &key in keys {
        if let Ok(val) = std::env::var(key) {
            if !val.is_empty() {
                return val;
            }
        }
    }
    default.to_string()
}

pub fn should_skip_integration_test() -> bool {
    // Skip if no API token is provided (indicates no server available)
    let token = get_env_var(&["VIKUNJA_TOKEN", "VIKUNJA_API_TOKEN"], "");
    if token.is_empty() {
        return true;
    }
    
    // Skip if SKIP_INTEGRATION_TESTS is set
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return true;
    }
    
    false
}

/// Event simulation helpers for realistic testing
pub struct KeyEventSimulator {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyEventSimulator {
    pub fn new() -> Self {
        Self { ctrl: false, alt: false, shift: false }
    }
    
    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
    
    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }
    
    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }
    
    pub fn create_event(&self, code: KeyCode) -> KeyEvent {
        let mut modifiers = KeyModifiers::empty();
        if self.ctrl { modifiers |= KeyModifiers::CONTROL; }
        if self.alt { modifiers |= KeyModifiers::ALT; }
        if self.shift { modifiers |= KeyModifiers::SHIFT; }
        
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }
}

/// Helper to create a test app with tasks and quick actions
pub fn create_test_app_with_keybindings() -> App {
    let mut config = CriaConfig::default();
    config.quick_actions = Some(vec![
        QuickAction {
            key: "w".to_string(),
            action: "project".to_string(),
            target: "Work".to_string(),
        },
        QuickAction {
            key: "h".to_string(),
            action: "priority".to_string(),
            target: "5".to_string(),
        },
        QuickAction {
            key: "u".to_string(),
            action: "label".to_string(),
            target: "urgent".to_string(),
        },
    ]);
    
    let mut app = App::new_with_config(config, "Inbox".to_string());
    
    // Add some test tasks
    app.all_tasks = vec![
        sample_task(1, false),
        sample_task(2, true),
        sample_task(3, false),
    ];
    app.tasks = app.all_tasks.clone();
    
    // Add test projects  
    app.project_map.insert(1, "Inbox".to_string());
    app.project_map.insert(2, "Work".to_string());
    app.project_map.insert(3, "Personal".to_string());
    
    // Add test labels
    app.label_map.insert(1, "urgent".to_string());
    app.label_map.insert(2, "low-priority".to_string());
    
    app
}

/// Create a sample task for testing
pub fn sample_task(id: i64, done: bool) -> Task {
    Task {
        id,
        title: format!("Test Task {}", id),
        done,
        is_favorite: false,
        labels: Some(vec![]),
        assignees: None,
        project_id: 1,
        priority: Some(1),
        due_date: Some(Utc.from_utc_datetime(&NaiveDate::from_ymd_opt(2025, 12, 31).unwrap().and_hms_opt(0,0,0).unwrap())),
        start_date: None,
        description: Some("Test description".to_string()),
        done_at: None,
        end_date: None,
        created: None,
        updated: None,
        created_by: None,
        percent_done: None,
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
    }
}

/// Create a simple key event without modifiers
pub fn create_key_event(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    }
}

/// Create a test app with minimal setup
pub fn create_minimal_test_app() -> App {
    let config = CriaConfig::default();
    App::new_with_config(config, "Test Project".to_string())
}

/// Create a test app with specific number of tasks
pub fn create_test_app_with_tasks(task_count: usize) -> App {
    let mut app = create_test_app_with_keybindings();
    app.all_tasks = (1..=task_count)
        .map(|i| sample_task(i as i64, i % 2 == 0))
        .collect();
    app.tasks = app.all_tasks.clone();
    app
}
