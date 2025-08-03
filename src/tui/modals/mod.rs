mod quick_add;
mod edit;
mod form_edit;
mod attachments;
mod file_picker;
mod comments;
// Relations - DISABLED: Incomplete feature
// mod relations;

pub use quick_add::handle_quick_add_modal;
pub use edit::handle_edit_modal;
pub use form_edit::handle_form_edit_modal;
pub use attachments::{AttachmentModal, AttachmentModalAction};
pub use file_picker::{FilePickerModal, FilePickerAction, FileEntry};
pub use comments::{CommentsModal, CommentsModalAction};
// Relations - DISABLED: Incomplete feature  
// pub use relations::{handle_relations_modal, handle_add_relation_modal};