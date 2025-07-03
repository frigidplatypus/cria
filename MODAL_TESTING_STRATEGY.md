# CRIA Modal Testing Strategy

This document outlines the comprehensive testing strategies implemented in CRIA for testing TUI modals without running the full GUI, as well as automated testing approaches.

## Testing Approaches Implemented

### 1. Unit Testing for Modal State Logic (`tests/modals.rs` & `tests/modals_comprehensive.rs`)

**What it tests:**
- Modal opening/closing behavior
- Input handling (text entry, cursor movement, deletion)
- Modal exclusivity (only one modal open at a time)
- State preservation and reset behavior
- Quick action execution logic
- Error handling for invalid inputs

**Key features:**
- Tests modal state transitions directly on the App struct
- Validates input/output behavior without GUI rendering
- Tests interaction between different modals
- Covers edge cases and error conditions

**Example:**
```rust
#[test]
fn test_modal_exclusivity() {
    let mut app = create_test_app_with_quick_actions();
    
    // Test that only one modal can be open at a time
    app.show_quick_add_modal();
    assert!(app.show_quick_add_modal);
    
    app.show_edit_modal();
    assert!(!app.show_quick_add_modal);  // Previous modal should be closed
    assert!(app.show_edit_modal);
}
```

### 2. Mock Terminal Rendering Tests (`tests/ui_rendering.rs`)

**What it tests:**
- UI rendering functions work without panicking
- Modal rendering at different terminal sizes
- Content validation in rendered output
- Proper handling of edge cases (empty data, small terminals)

**Key features:**
- Uses `ratatui::backend::TestBackend` for mock terminal
- Tests actual UI drawing functions
- Validates rendered content
- Tests responsiveness to different screen sizes

**Example:**
```rust
#[test]
fn test_quick_actions_modal_rendering() {
    let mut terminal = create_test_terminal();
    let mut app = create_test_app_with_quick_actions();
    app.show_quick_actions_modal();
    
    // Test that rendering doesn't panic
    terminal.draw(|f| {
        draw_quick_actions_modal(f, &app);
    }).unwrap();
}
```

### 3. Integration Testing (`tests/app.rs`)

**What it tests:**
- Full app workflow integration
- Config loading and validation
- Task management with modal interactions
- Quick actions integration with the app state

**Key features:**
- Tests end-to-end workflows
- Validates config file handling
- Tests interaction between modals and main app functionality

## Testing Strategies for Different Modal Types

### Quick Actions Modal
- **State Testing**: Navigation between actions, selection handling
- **Execution Testing**: Action application to tasks (project change, priority, labels)
- **Error Testing**: Invalid targets, no tasks available
- **UI Testing**: Colorized display, keyboard navigation rendering

### Quick Add Modal
- **Input Testing**: Text entry, cursor movement, deletion
- **State Testing**: Modal opening/closing, input reset
- **Integration Testing**: Task creation from modal input

### Edit Modal
- **Input Testing**: Text editing, cursor positioning
- **State Testing**: Pre-population with existing task data
- **Integration Testing**: Task modification and persistence

### Help Modal
- **Content Testing**: Display of keybinds and quick actions
- **Responsiveness**: Adapting to different terminal sizes
- **Config Integration**: Showing current config information

### Sort Modal
- **State Testing**: Selection navigation, sort option handling
- **Integration Testing**: Applying sort to task list
- **UI Testing**: Display of available sort options

## Advanced Testing Strategies

### 1. Property-Based Testing
Could be implemented for:
- Random input sequences to modal inputs
- Fuzzing modal state transitions
- Testing with various config combinations

### 2. Snapshot Testing
Could be implemented for:
- Capturing rendered modal output
- Comparing UI changes over time
- Regression testing for layout changes

### 3. End-to-End Testing with Event Simulation

For fully automated GUI testing, you could implement:

```rust
// Example approach using crossterm event simulation
#[test]
fn test_modal_workflow_e2e() {
    // This would require more complex setup with event simulation
    let mut app = App::new();
    
    // Simulate key presses
    simulate_key_press(&mut app, KeyCode::Char(' '));  // Open quick actions
    simulate_key_press(&mut app, KeyCode::Down);       // Navigate
    simulate_key_press(&mut app, KeyCode::Enter);      // Select action
    
    // Verify expected state changes
    assert!(/* expected behavior */);
}
```

### 4. Testing with External Libraries

**For GUI automation (more complex):**
- `expectrl` - Terminal automation
- Custom event simulation with crossterm
- Mock terminal sessions

**Example with expectrl (if implemented):**
```rust
#[test]
fn test_full_gui_automation() {
    let mut session = expectrl::spawn("./target/debug/cria").unwrap();
    session.expect("CRIA - Task Manager").unwrap();
    session.send(" ").unwrap();  // Open quick actions
    session.expect("Quick Actions").unwrap();
    // ... more automation
}
```

## Current Test Coverage

### Modal State Logic: ✅ Comprehensive
- All modal types covered
- State transitions tested
- Error conditions handled
- Input/output validation

### UI Rendering: ✅ Good
- Mock terminal testing implemented
- Different screen sizes tested
- Basic content validation
- Panic-free rendering verified

### Integration: ✅ Good
- Config integration tested
- Modal-to-app-state integration covered
- Quick actions fully tested

### End-to-End: ⚠️ Limited
- Currently relies on manual testing
- Could be enhanced with event simulation
- GUI automation not implemented

## Recommendations for Future Enhancement

1. **Add Property-Based Testing**
   - Use `proptest` or `quickcheck` for random input testing
   - Test modal behavior with various input sequences

2. **Implement Event Simulation Framework**
   - Create helper functions for simulating key presses
   - Build reusable test scenarios

3. **Add Snapshot Testing**
   - Capture and compare rendered output
   - Detect unintended UI changes

4. **Enhance Integration Testing**
   - Test more complex workflows
   - Add performance testing for large datasets

5. **Add Accessibility Testing**
   - Test keyboard navigation completeness
   - Verify screen reader compatibility

## Running the Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test modals                    # Basic modal tests
cargo test --test modals_comprehensive      # Advanced modal tests  
cargo test --test ui_rendering              # UI rendering tests
cargo test --test app                       # Integration tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_modal_exclusivity
```

## Test Development Guidelines

1. **Unit Tests**: Test individual modal functions in isolation
2. **Integration Tests**: Test modal interaction with app state
3. **UI Tests**: Use TestBackend for rendering validation
4. **Error Tests**: Always test error conditions and edge cases
5. **State Tests**: Verify modal state transitions and cleanup
6. **Performance Tests**: Consider testing with large datasets

This comprehensive testing strategy ensures that CRIA's modal functionality is robust, reliable, and maintainable without requiring manual GUI testing for most scenarios.
