// Centralized keybinding definitions for CRIA TUI

use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    Quit,
    MoveDown,
    MoveUp,
    JumpTop,
    JumpBottom,
    EditTask,
    AddTask,
    ToggleComplete,
    DeleteTask,
    CompleteTask,
    Undo,
    ToggleDebug,
    ShowKeybinds,
    ShowFilterPicker,
    ShowProjectPicker,
    CloseModal,
    CycleFilterBackward,
    CycleFilterForward,
    ToggleInfoPane,
    Refresh,
    ToggleStar,
    // ...add more as needed
}

#[derive(Debug, Clone)]
pub struct Keybind {
    pub key: &'static str,
    pub description: &'static str,
    pub action: KeyAction,
    pub keycodes: &'static [KeyCode],
}

pub const KEYBINDS: &[Keybind] = &[
    Keybind { key: "q", description: "Quit", action: KeyAction::Quit, keycodes: &[KeyCode::Char('q')] },
    Keybind { key: "j / Down", description: "Move down", action: KeyAction::MoveDown, keycodes: &[KeyCode::Char('j'), KeyCode::Down] },
    Keybind { key: "k / Up", description: "Move up", action: KeyAction::MoveUp, keycodes: &[KeyCode::Char('k'), KeyCode::Up] },
    Keybind { key: "g", description: "Jump to top", action: KeyAction::JumpTop, keycodes: &[KeyCode::Char('g')] },
    Keybind { key: "G", description: "Jump to bottom", action: KeyAction::JumpBottom, keycodes: &[KeyCode::Char('G')] },
    Keybind { key: "e / Enter", description: "Edit task", action: KeyAction::EditTask, keycodes: &[KeyCode::Char('e'), KeyCode::Enter] },
    Keybind { key: "a", description: "Add task", action: KeyAction::AddTask, keycodes: &[KeyCode::Char('a')] },
    Keybind { key: "d", description: "Toggle complete", action: KeyAction::ToggleComplete, keycodes: &[KeyCode::Char('d')] },
    Keybind { key: "D", description: "Delete task", action: KeyAction::DeleteTask, keycodes: &[KeyCode::Char('D')] },
    Keybind { key: "c", description: "Complete task", action: KeyAction::CompleteTask, keycodes: &[KeyCode::Char('c')] },
    Keybind { key: "u", description: "Undo last action", action: KeyAction::Undo, keycodes: &[KeyCode::Char('u')] },
    Keybind { key: "I", description: "Toggle debug modal", action: KeyAction::ToggleDebug, keycodes: &[KeyCode::Char('I')] },
    Keybind { key: "?", description: "Show keybinds (this help)", action: KeyAction::ShowKeybinds, keycodes: &[KeyCode::Char('?')] },
    Keybind { key: "f", description: "Show filter picker", action: KeyAction::ShowFilterPicker, keycodes: &[KeyCode::Char('f')] },
    Keybind { key: "p", description: "Show project picker", action: KeyAction::ShowProjectPicker, keycodes: &[KeyCode::Char('p')] },
    Keybind { key: "Esc", description: "Close modal/picker", action: KeyAction::CloseModal, keycodes: &[KeyCode::Esc] },
    Keybind { key: "h", description: "Cycle filter backward", action: KeyAction::CycleFilterBackward, keycodes: &[KeyCode::Char('h')] },
    Keybind { key: "l", description: "Cycle filter forward", action: KeyAction::CycleFilterForward, keycodes: &[KeyCode::Char('l')] },
    Keybind { key: "i", description: "Toggle info pane", action: KeyAction::ToggleInfoPane, keycodes: &[KeyCode::Char('i')] },
    Keybind { key: "r", description: "Refresh", action: KeyAction::Refresh, keycodes: &[KeyCode::Char('r')] },
    Keybind { key: "s", description: "Toggle star", action: KeyAction::ToggleStar, keycodes: &[KeyCode::Char('s')] },
    // ...add more as needed
];

pub fn action_for_keycode(keycode: &KeyCode) -> Option<KeyAction> {
    for kb in KEYBINDS {
        if kb.keycodes.contains(keycode) {
            return Some(kb.action);
        }
    }
    None
}
