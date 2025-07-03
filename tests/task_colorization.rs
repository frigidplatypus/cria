// Tests for colorization of project and labels in the TUI task list

use cria::config::CriaConfig;
use cria::tui::app::App;
use cria::tui::ui::main::draw;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::style::Color;

fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 20);
    Terminal::new(backend).unwrap()
}

#[test]
fn test_project_colorization_in_task_list() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.project_map.insert(1, "Inbox".to_string());
    app.project_colors.insert(1, "#ff0000".to_string()); // Red
    let mut task = cria::vikunja::models::Task::default();
    task.id = 1;
    task.title = "Color Test".to_string();
    task.project_id = 1;
    app.tasks = vec![task];
    let mut terminal = create_test_terminal();
    terminal.draw(|f| {
        draw(f, &app);
    }).unwrap();
    let buffer = terminal.backend().buffer();
    let found = buffer.content().iter().any(|cell| cell.fg == Color::Rgb(255, 0, 0));
    assert!(found, "Project color #ff0000 (red) should be used in task list");
}

#[test]
fn test_label_colorization_in_task_list() {
    let mut app = App::new_with_config(CriaConfig::default(), "Inbox".to_string());
    app.label_map.insert(1, "Urgent".to_string());
    app.label_colors.insert(1, "#00ff00".to_string()); // Green
    let label = cria::vikunja::models::Label {
        id: 1,
        title: "Urgent".to_string(),
        hex_color: Some("#00ff00".to_string()),
        description: None,
        created: None,
        updated: None,
        created_by: None,
    };
    let mut task = cria::vikunja::models::Task::default();
    task.id = 2;
    task.title = "Label Color Test".to_string();
    task.labels = Some(vec![label]);
    app.tasks = vec![task];
    let mut terminal = create_test_terminal();
    terminal.draw(|f| {
        draw(f, &app);
    }).unwrap();
    let buffer = terminal.backend().buffer();
    let found = buffer.content().iter().any(|cell| cell.fg == Color::Rgb(0, 255, 0));
    assert!(found, "Label color #00ff00 (green) should be used in task list");
}
