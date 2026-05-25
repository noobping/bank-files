use crate::model::{AppData, DedupeMode, TransactionLoadScope};

pub(super) fn load_search_data() -> AppData {
    crate::data::load_app_data_read_only_aware(
        DedupeMode::Enabled,
        false,
        TransactionLoadScope::All,
    )
    .map(|(data, _capabilities)| data)
    .unwrap_or_else(|err| {
        eprintln!("Failed to load transactions for GNOME search: {err}");
        AppData::default()
    })
}
