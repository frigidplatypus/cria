impl crate::tui::app::state::App {
    /// Show the quit confirmation dialog
    pub fn confirm_quit(&mut self) {
        self.show_confirmation_dialog = true;
        self.confirmation_message = "Are you sure you want to quit? (y/n)".to_string();
        self.pending_action = Some(crate::tui::app::pending_action::PendingAction::QuitApp);
    }
}
