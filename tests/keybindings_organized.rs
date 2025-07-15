// Keybinding tests organized into focused modules
// 
// This replaces the large monolithic keybindings.rs file with
// well-organized, focused test modules for better maintainability.

mod common;
mod keybindings;

// Re-export for backwards compatibility
pub use keybindings::*;
