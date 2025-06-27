# Refactoring TODO for CRIA TUI

## 1. UI Drawing Components
- Break up `tui/ui.rs` into smaller files:
  - `tui/ui/main.rs` (main layout)
  - `tui/ui/task_list.rs` (task list rendering)
  - `tui/ui/task_details.rs` (task details pane)
  - `tui/ui/modals.rs` (quick add/edit modal rendering)
  - `tui/ui/pickers.rs` (project/filter picker rendering)
- **Benefit:** Each UI component is easier to reason about and update independently.

## 2. App State and Domain Logic
- Split `tui/app.rs` into:
  - `tui/app/state.rs` (core App struct and state)
  - `tui/app/filters.rs` (filtering logic)
  - `tui/app/projects.rs` (project logic)
  - `tui/app/tasks.rs` (task manipulation logic)
- **Benefit:** Domain logic is separated from UI and event handling.

## 3. API Client
- Split `vikunja_client.rs` by resource: **(DONE)**
  - `vikunja_client/tasks.rs` **(DONE)**
  - `vikunja_client/projects.rs` **(DONE)**
  - `vikunja_client/filters.rs` **(DONE)**
  - `vikunja_client/users.rs` **(DONE)**
- **Benefit:** Each API area is easier to test and extend.

## 4. Event Handling
- If event handler files grow, split by modal/picker type:
  - `tui/modals/quick_add.rs`
  - `tui/modals/edit.rs`
  - `tui/pickers/project.rs`
  - `tui/pickers/filter.rs`
- **Benefit:** Each handler is focused and easier to maintain.

## 5. Helpers and Utilities
- Create a `tui/utils.rs` or `tui/helpers.rs` for:
  - Color utilities
  - String normalization
  - Fuzzy matching
  - Logging helpers
- **Benefit:** Reduces duplication and centralizes helpers.

## 6. Testing
- Add a `tests/` directory or module-level tests in each component.
- **Benefit:** Easier to test and maintain code quality.

---

**Summary:**
As the app grows, breaking down by UI component, domain logic, API resource, and event handler type will keep the codebase clean and scalable. Prioritize areas that are growing or hard to maintain first.
