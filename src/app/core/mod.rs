use super::*;

mod availability;
mod build;
mod i18n;
mod loading;
mod navigation;
mod preference_sync;
mod scope;
mod session;
mod startup;
mod types;

pub(in crate::app) use availability::{
    apply_action_availability, config_write_availability, data_write_availability,
    refresh_write_actions, register_loading_sensitive_widget, set_storage_capabilities,
    ActionAvailability,
};
pub(in crate::app) use build::{build_ui, build_ui_with_opened_uris};
pub(in crate::app) use i18n::{tr, trf};
pub(in crate::app) use loading::{begin_background_operation, finish_background_operation};
pub(in crate::app) use navigation::navigate_back;
pub(in crate::app) use scope::{comparison_mode, current_transaction_load_scope};
pub(in crate::app) use session::ACTIVE_SESSION;
pub use startup::run;
pub(in crate::app) use types::UiHandles;

pub(in crate::app) const CATEGORY_PREVIEW_LIMIT: usize = 5;
pub(in crate::app) const SEARCH_CATEGORY_PREVIEW_LIMIT: usize = 6;
