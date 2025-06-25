use crossterm::event::{KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::stdout;
use std::sync::Arc;
use tokio::sync::Mutex;

mod tui;
mod vikunja;

use crate::tui::app::App;
use crate::tui::events::{Event, EventHandler};
use crate::tui::ui::draw;
use crate::vikunja::client::VikunjaClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let client = VikunjaClient::new();
    let app = Arc::new(Mutex::new(App::new()));
    let event_handler = EventHandler::new(250);

    let app_clone = app.clone();
    let client_clone = client.clone();

    // Load tasks and projects before starting the UI
    let (tasks, project_map, project_colors) = client_clone.get_tasks_with_projects().await.unwrap_or_default();
    {
        let mut app_guard = app.lock().await;
        app_guard.tasks = tasks;
        app_guard.project_map = project_map;
        app_guard.project_colors = project_colors;
    }

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    loop {
        let app_guard = app.lock().await;
        terminal.draw(|frame| draw(frame, &app_guard))?;
        drop(app_guard);

        if let Event::Key(key) = event_handler.next()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        let mut app_guard = app.lock().await;
                        app_guard.quit();
                    },
                    _ => {},
                }
            }
        }

        let app_guard = app.lock().await;
        if !app_guard.running {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
