use crate::tui::app::App;
use crate::tui::events::Event;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::vikunja_client::VikunjaClient;

// Handler for the main event loop keybindings
// (Currently not used, but kept for future modular event handling)
pub async fn handle_main_event(
    _app: &Arc<Mutex<App>>,
    _api_client: &Arc<Mutex<VikunjaClient>>,
    _client_clone: &Arc<Mutex<VikunjaClient>>,
    _event: &Event,
) -> Option<bool> {
    None
}
