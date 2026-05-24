use crate::model::{AppData, DedupeMode, TransactionLoadScope};

#[cfg(feature = "smart-insights")]
use crate::app_info::APP_ID;
#[cfg(feature = "smart-insights")]
use adw::gio::{self, prelude::SettingsExt};

pub(super) fn load_search_data() -> AppData {
    crate::data::load_app_data_read_only_aware(
        DedupeMode::Enabled,
        false,
        TransactionLoadScope::All,
        search_smart_insights_enabled(),
    )
    .map(|(data, _capabilities)| data)
    .unwrap_or_else(|err| {
        eprintln!("Failed to load transactions for GNOME search: {err}");
        AppData::default()
    })
}

#[cfg(feature = "smart-insights")]
fn search_smart_insights_enabled() -> bool {
    gio::SettingsSchemaSource::default()
        .and_then(|source| source.lookup(APP_ID, true))
        .map(|_| gio::Settings::new(APP_ID).boolean("show-predictions"))
        .unwrap_or(false)
}

#[cfg(not(feature = "smart-insights"))]
fn search_smart_insights_enabled() -> bool {
    false
}
