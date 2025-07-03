# CRIA TUI Development Roadmap

## 🎯 **Current Status: v0.8.0 (Beta Release)**

CRIA TUI is feature-complete for core task management functionality with a clean, well-tested codebase. Ready for user testing and feedback-driven improvements.

### ✅ **Completed (v0.8.0)**
- Complete TUI task management (create, edit, delete, complete)
- Auto-suggestion system with multi-word support
- Square bracket parsing for labels/projects/assignees
- Project and filter management
- Comprehensive test coverage
- Zero compiler warnings
- Clean, modular architecture

### ✅ **Completed (v0.8.5)**
- **Undo/Redo System** - Implemented comprehensive undo/redo functionality
  - `Ctrl+Z` / `Ctrl+Y` keybindings in main view and modals
  - Visual feedback for undo/redo operations
  - Support for task completion, deletion, creation, and editing
  - Bounded stacks (50 items) with automatic cleanup
  - Full test coverage with 13+ unit tests

### ✅ **Completed (v0.8.6)**
- **Flexible Column System with Auto-Width** 📊
  - Due date and start date columns added to task list
  - Automatic column width calculation based on content and available space
  - Text wrapping for long task titles and labels
  - Named column layouts (switch with `Shift+H`/`Shift+L`)
  - User-configurable min/max widths and wrap settings
  - Multiple predefined layouts: default, minimal, project-focused, time-management
  - Backward compatible with legacy column configuration
### **v0.9.0 - User Experience & Quality of Life** 
*Target: After initial user testing feedback*

#### **High-Priority Features:**
- **Global Task Search** 🔍
  - Press `/` to open search mode
  - Search across task titles, descriptions, labels
  - Real-time filtering as you type
  - Saved search queries

#### **User Experience Improvements:**
- **Enhanced Visual Feedback** 🎨
  - Progress bars for loading states
  - Toast notifications for successful actions
  - Task priority color coding
  - Overdue task highlighting (red text/background)

- **Better Task Details** 📝
  - View/edit task descriptions in modal
  - Due date picker with calendar widget
  - Show task creation/modified dates

### **v0.10.0 - Power User Features**
*Target: Based on user feedback and feature requests*

#### **Bulk Operations** ⚡
- Multi-select tasks (`Space` to toggle, `Shift+arrows`)
- Bulk complete/delete/move operations
- Batch label assignment
- Mass project reassignment

#### **Quick Actions** ⚡
- Command palette (`:` key like Vim)
- Quick task templates
- Bulk edit mode
- Keyboard shortcuts help (`Ctrl+?`)

#### **Data Management** 💾
- Export tasks to JSON/CSV
- Import from other todo apps (Todoist, Things, etc.)
- Backup/restore functionality
- Basic offline mode with sync

### **v1.0.0 - Stable Release**
*Target: After extensive user testing and feedback*

#### **Requirements for v1.0:**
- ✅ All major bugs resolved
- ✅ User feedback incorporated
- ✅ Performance optimized for large task lists
- ✅ Comprehensive documentation
- ✅ Stable API/config format
- ✅ Production deployment guide

---

## 🔮 **Future Considerations (v1.1+)**

### **Advanced Features:**
- **Task Dependencies** 🔗
  - Subtask hierarchy
  - Blocking/blocked relationships
  - Dependency visualization

- **Integrations** 🔗
  - CLI companion tool (`cria-cli`)
  - Desktop notifications
  - System tray integration
  - Webhook support for automation

- **Collaboration** 👥
  - Task assignment to team members
  - Comments and activity feeds
  - Real-time sync across clients

### **Performance & Scale:**
- **Optimization** ⚡
  - Lazy loading for large task lists (1000+ tasks)
  - Background sync with progress indicators
  - Memory usage optimization
  - Database-level caching

### **Platform Expansion:**
- **Cross-Platform** 🌐
  - Windows/macOS native binaries
  - Web interface (WASM build)
  - Mobile companion app
  - Browser extension

---

## 🎯 **Development Priorities**

### **Phase 1: User Testing Readiness (v0.8.0)**
- ✅ **COMPLETE** - Feature-complete core functionality
- ✅ **COMPLETE** - Clean, bug-free codebase
- ✅ **COMPLETE** - Comprehensive testing

### **Phase 2: User Experience (v0.9.0)**
Focus on making the app delightful to use:
1. **Undo/Redo** - Critical for user confidence
2. **Search** - Essential for productivity
3. **Configuration** - Needed for adoption
4. **Visual Polish** - Professional feel

### **Phase 3: Power Features (v0.10.0)**
Focus on advanced workflows:
1. **Bulk Operations** - Handle large task lists
2. **Quick Actions** - Power user efficiency
3. **Data Portability** - User data ownership

### **Phase 4: Stability (v1.0.0)**
Focus on production readiness:
1. **Performance** - Handle real-world usage
2. **Documentation** - Easy onboarding
3. **Deployment** - Simple installation

---

## 🔍 **User Testing Focus Areas**

### **Key Questions for Beta Users:**
1. **Core Workflow:** Is the basic task management intuitive?
2. **Auto-Suggestions:** Do the label/project suggestions help or hinder?
3. **Keybindings:** Are the keyboard shortcuts discoverable/memorable?
4. **Performance:** How does it feel with 50+ tasks? 200+ tasks?
5. **Missing Features:** What would make this indispensable for daily use?

### **Success Metrics:**
- Users can create/complete 10 tasks without documentation
- Average time to complete common actions
- Feature usage analytics (which features are ignored?)
- Bug reports and crash frequency
- User retention after first week

---

## 🤝 **Contributing**

### **How to Contribute:**
1. **User Testing:** Try the app with your real tasks
2. **Feature Requests:** Open issues with detailed use cases
3. **Bug Reports:** Include reproduction steps and environment
4. **Code Contributions:** See architecture docs in `/docs`

### **Development Setup:**
```bash
git clone https://github.com/username/cria
cd cria
cargo build --release
cargo test
./target/release/cria --dev-env
```

### **Architecture Overview:**
- **Modular Design:** UI, API, State cleanly separated
- **Event-Driven:** Each modal/picker has focused event handlers
- **Tested:** Comprehensive test coverage for core functionality
- **Documented:** Auto-suggestion and quick-add guides available

---

## 📅 **Timeline Estimates**

| Version | Target Date | Focus | Key Features |
|---------|-------------|-------|--------------|
| v0.8.0 | ✅ **Current** | Core Functionality | Task CRUD, Auto-suggestions, Clean Architecture |
| v0.9.0 | +2 months | User Experience | Undo/Redo, Search, Configuration, Visual Polish |
| v0.10.0 | +4 months | Power Features | Bulk Operations, Quick Actions, Data Export |
| v1.0.0 | +6 months | Stable Release | Performance, Documentation, Production Ready |

*Note: Timeline depends on user feedback volume and contributor availability*

---

## 💭 **Philosophy & Design Principles**

### **Core Values:**
- **Keyboard-First:** Mouse optional, everything accessible via keyboard
- **Fast & Responsive:** No waiting, immediate feedback
- **Privacy-Focused:** Your data stays on your Vikunja instance
- **Unix Philosophy:** Do one thing well, integrate with other tools
- **Rust Quality:** Memory safe, fast, reliable

### **Design Decisions:**
- **TUI over GUI:** Terminal-native for developer workflows
- **Vikunja Integration:** Leverage existing robust backend
- **Modal Interface:** Vim-inspired for efficiency
- **Auto-Suggestions:** Reduce typing, increase consistency
- **Test-Driven:** Changes backed by automated tests

---

*Last Updated: January 2025*
*Next Review: After v0.8.0 user testing phase*
