/// Modal subsystem for TUI. All modal openers must use the centralized viewport guard in `utils.rs`.
/// This ensures modals do not open or crash in undersized terminals, and instead show a user message.
///
/// Maintainability: Modal draw functions should not contain viewport size checks. All guards are centralized.
/// See `try_show_modal` in `utils.rs` for details and constants.
pub mod utils;

mod quick_add;
mod edit;
mod form_edit;
mod attachments;
mod file_picker;
mod comments;
pub mod url_modal;
// Relations - DISABLED: Incomplete feature
// mod relations;


pub use quick_add::handle_quick_add_modal;
pub use edit::handle_edit_modal;
pub use form_edit::handle_form_edit_modal;
pub use attachments::{AttachmentModal, AttachmentModalAction};
pub use file_picker::{FilePickerModal, FilePickerAction};
pub use comments::{CommentsModal, CommentsModalAction};
pub use url_modal::{UrlModal, UrlModalAction};
// Relations - DISABLED: Incomplete feature  
// pub use relations::{handle_relations_modal, handle_add_relation_modal};