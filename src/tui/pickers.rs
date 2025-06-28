//! Picker event handlers for the CRIA TUI application.
//!
//! This module is split by picker type for maintainability:
//!
//! - `project`: Project picker event handler
//! - `filter`: Filter picker event handler
//!
//! Each handler is in its own file under `pickers/` and re-exported here.
//!
//! Update this module if new picker types are added or split.

pub mod project;
pub mod filter;

pub use project::handle_project_picker;
pub use filter::handle_filter_picker;
