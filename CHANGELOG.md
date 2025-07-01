# Changelog

All notable changes to CRIA TUI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2025-07-01

### 🎉 Major Release - Production Ready Beta

This is the first major release of CRIA TUI, featuring a complete refactor and comprehensive feature set for task management with Vikunja.

### ✨ Features

#### Core Functionality
- **Complete TUI task management** - Create, edit, delete, and complete tasks
- **Project management** - Switch between projects, create new projects
- **Filter management** - Apply and manage custom filters
- **Auto-suggestion system** - Intelligent suggestions for labels, projects, and assignees
- **Square bracket parsing** - Support for `[label]`, `@[project]`, and `+[assignee]` syntax

#### Enhanced User Experience
- **Multi-word auto-suggestions** - Support for complex label and project names
- **Fuzzy matching** - Smart matching with scoring for better suggestions
- **Visual feedback** - Clear UI states and interactive elements
- **Keyboard-driven workflow** - Complete mouse-free operation

#### Technical Excellence
- **Modular architecture** - Clean separation of UI, state, and API components
- **Comprehensive test coverage** - Tests for modals, pickers, and parsing logic
- **Zero compiler warnings** - Production-ready, clean codebase
- **Performance optimized** - Efficient rendering and state management

### 🔧 Technical Improvements

#### Architecture Refactoring
- Split monolithic files into focused modules:
  - `tui/ui/` - UI rendering components
  - `tui/app/` - Application state and domain logic
  - `tui/modals/` - Modal event handlers
  - `tui/pickers/` - Picker event handlers
  - `vikunja_client/` - API client by resource type

#### Code Quality
- **Event handling refactor** - Dedicated handlers for each modal and picker type
- **Utility centralization** - Common functions in `tui/utils.rs`
- **Error handling** - Robust error handling throughout the application
- **Memory safety** - Leveraging Rust's ownership system for reliability

### 🐛 Bug Fixes
- **Task completion sync** - Fixed "d" key bug where task completion only updated UI
- **Key binding conflicts** - Resolved star '*' vs sort 's' key conflicts
- **Visual rendering issues** - Fixed suggestion insertion bugs in add/edit modal
- **Parser edge cases** - Improved handling of various delimiter styles

### 📚 Documentation
- **Auto-suggestion guide** - Comprehensive `AUTO_SUGGESTION_DEMO.md`
- **Quick-add documentation** - Updated `README-QUICKADD.md` with examples
- **Development roadmap** - Detailed `ROADMAP.md` for future planning
- **Architecture docs** - Clear module organization and responsibilities

### 🧪 Testing
- **Modal event testing** - Comprehensive tests for modal interactions
- **Picker event testing** - Complete picker behavior validation
- **Parser testing** - Edge case coverage for square bracket parsing
- **Integration tests** - API operation validation

### 🔄 Breaking Changes
- **File structure** - Significant reorganization of source files
- **Event handling** - New event handler architecture (internal change)

### 🎯 What's Next (v0.9.0)
- Undo/Redo system
- Global task search
- Configuration system (themes, keybindings)
- Enhanced visual feedback

---

## Version History

### [0.8.0] - 2025-07-01
- Initial major release
- Complete TUI task management functionality
- Auto-suggestion system with multi-word support
- Clean, modular architecture
- Comprehensive test coverage
- Zero compiler warnings

---

*For detailed development history and future plans, see [ROADMAP.md](ROADMAP.md)*
