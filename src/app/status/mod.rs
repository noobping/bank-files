use super::*;

mod bar;
mod feedback;
mod history;
mod history_rows;
mod lifecycle;
mod page_actions;
mod snapshot;

pub(in crate::app) use bar::{build_status_bar, StatusBar, StatusHandle};
pub(in crate::app) use feedback::{register_page_copy_feedback_button, show_page_copy_feedback};
pub(in crate::app) use history::StatusLogEntry;
pub(in crate::app) use lifecycle::{
    connect_embedded_status_bar, connect_status_actions, schedule_status_autohide_after_loading,
    show_status,
};
pub(in crate::app) use page_actions::{connect_page_actions, connect_static_page_actions};
pub(in crate::app) use snapshot::{PageActionSnapshot, StaticPageSnapshot};

pub(super) const STATUS_AUTOHIDE_SECONDS: u32 = 6;
pub(super) const COPY_FEEDBACK_SECONDS: u32 = 3;
pub(super) const COPY_ICON: &str = "edit-copy-symbolic";
pub(super) const COPIED_ICON: &str = "object-select-symbolic";

#[cfg(test)]
mod tests;
