//! Modal event handlers for the CRIA TUI application.
//!
//! This module is split by modal type for maintainability:
//!
//! - `quick_add`: Quick Add modal event handler
//! - `edit`: Edit modal event handler
//!
//! Each handler is in its own file under `modals/` and re-exported here.
//!
//! Update this module if new modal types are added or split.

pub mod quick_add;
pub mod edit;

pub use quick_add::handle_quick_add_modal;
pub use edit::handle_edit_modal;
