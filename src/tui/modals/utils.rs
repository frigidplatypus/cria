
/// Modal utilities: centralized viewport guard and constants for modal maintainability.
///
/// All modal openers must use `try_show_modal` to check viewport size before opening.
/// If the terminal is too small, a toast is shown and the modal is not opened.
///
/// Maintainability: Do not duplicate viewport checks in modal draw functions. All guards are centralized here.
///
/// Usage:
/// ```rust
/// use crate::tui::modals::utils::try_show_modal;
/// if try_show_modal(app, terminal, |app| app.show_edit_modal()) {
///     // Modal opened
/// }
/// ```
///
/// Constants:
/// - MIN_MODAL_WIDTH: Minimum width for any modal
/// - MIN_MODAL_HEIGHT: Minimum height for any modal
///
/// See mod.rs for subsystem documentation.

use ratatui::prelude::{CrosstermBackend, Terminal};
use crate::tui::app::state::App;

pub const MIN_MODAL_WIDTH: u16 = 40;
pub const MIN_MODAL_HEIGHT: u16 = 10;

/// Checks viewport size before opening a modal. If too small, shows a toast and returns false.
pub fn try_show_modal<F>(app: &mut App, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>, modal_fn: F) -> bool
where
    F: FnOnce(&mut App),
{
    let size = terminal.size().unwrap();
    if size.width < MIN_MODAL_WIDTH || size.height < MIN_MODAL_HEIGHT {
        app.show_toast("Viewport too small to display modal".to_string());
        false
    } else {
        modal_fn(app);
        true
    }
}
