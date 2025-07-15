use cria::tui::app::state::App;
use cria::tui::app::pending_action::PendingAction;

#[test]
fn test_show_and_cancel_confirmation_dialog() {
    let mut app = App::default();
    app.confirmation_message = String::new();
    app.show_confirmation_dialog = false;
    app.pending_action = None;

    // Simulate showing a confirmation dialog for deleting a task
    app.confirmation_message = "Delete task?".to_string();
    app.show_confirmation_dialog = true;
    app.pending_action = Some(PendingAction::DeleteTask { task_id: 1 });
    assert!(app.show_confirmation_dialog);
    assert_eq!(app.confirmation_message, "Delete task?");
    assert!(matches!(app.pending_action, Some(PendingAction::DeleteTask { .. })));

    // Simulate cancel
    app.show_confirmation_dialog = false;
    app.pending_action = None;
    assert!(!app.show_confirmation_dialog);
    assert!(app.pending_action.is_none());
}
