// lib.rs - Library interface for cria

pub mod vikunja_client;
pub mod vikunja_parser;
pub mod vikunja;
pub mod tui;
pub mod debug;
pub mod config;
pub mod terminal_capabilities;
pub mod url_utils;

// Re-export commonly used items
pub use vikunja_client::*;
pub use vikunja_parser::*;
