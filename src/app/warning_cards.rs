use super::warnings::{attention_warning_card_message, AttentionWarning};
use super::{trf, AppData, TransactionLoadScope};
use crate::ui;
use adw::gtk;
use adw::prelude::*;

pub(in crate::app) fn append_attention_warning_card(
    container: &gtk::Box,
    warnings: &[AttentionWarning],
) {
    if let Some(message) = attention_warning_card_message(warnings) {
        container.append(&ui::warning_card("Check your budget", &message));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PartialLoadRecordCounts {
    loaded: usize,
    total: usize,
}

pub(in crate::app) fn append_partial_load_notice(container: &gtk::Box, data: &AppData) {
    let Some(counts) = partial_load_record_counts(data) else {
        return;
    };

    let message = trf(
        "{loaded} / {total} CSV records are loaded to keep this page fast. Use Reload All for a full forced reload; that can take some time.",
        &[
            ("loaded", counts.loaded.to_string()),
            ("total", counts.total.to_string()),
        ],
    );
    container.append(&ui::warning_card(
        "Only part of the CSV records is loaded",
        &message,
    ));
}

fn partial_load_record_counts(data: &AppData) -> Option<PartialLoadRecordCounts> {
    if matches!(
        data.loaded_scope,
        TransactionLoadScope::All | TransactionLoadScope::Unloaded
    ) {
        return None;
    }

    let loaded = data
        .reports
        .iter()
        .map(|report| report.loaded_records())
        .sum::<usize>();
    let total = data
        .reports
        .iter()
        .map(|report| report.total_records())
        .sum::<usize>();

    (total > loaded).then_some(PartialLoadRecordCounts { loaded, total })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ImportReport;

    fn report(loaded: usize, skipped: usize, total: usize) -> ImportReport {
        ImportReport {
            rows_imported: loaded,
            rows_skipped: skipped,
            records_total: total,
            ..ImportReport::default()
        }
    }

    #[test]
    fn partial_load_notice_uses_loaded_records_not_only_imported_rows() {
        let data = AppData {
            loaded_scope: TransactionLoadScope::Year(Some(2026)),
            reports: vec![report(7, 3, 20)],
            ..AppData::default()
        };

        assert_eq!(
            partial_load_record_counts(&data),
            Some(PartialLoadRecordCounts {
                loaded: 10,
                total: 20
            })
        );
    }

    #[test]
    fn partial_load_notice_is_hidden_for_all_or_complete_scopes() {
        let all_data = AppData {
            loaded_scope: TransactionLoadScope::All,
            reports: vec![report(1, 0, 10)],
            ..AppData::default()
        };
        let complete_partial_data = AppData {
            loaded_scope: TransactionLoadScope::Year(Some(2026)),
            reports: vec![report(8, 2, 10)],
            ..AppData::default()
        };

        assert_eq!(partial_load_record_counts(&all_data), None);
        assert_eq!(partial_load_record_counts(&complete_partial_data), None);
    }
}
