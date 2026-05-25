use super::warnings::{attention_warning_card_message, AttentionWarning};
use super::{register_loading_sensitive_widget, trf, AppData, TransactionLoadScope, UiHandles};
use crate::ui;
use adw::gtk;
use adw::prelude::*;
use std::rc::Rc;

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

pub(in crate::app) fn append_partial_load_notice(
    container: &gtk::Box,
    ui_handles: &Rc<UiHandles>,
    data: &AppData,
) {
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
    container.append(&partial_load_info_card(ui_handles, &message));
}

fn partial_load_info_card(ui_handles: &Rc<UiHandles>, message: &str) -> gtk::Box {
    let card = ui::card_container();
    let content = ui::card_content(gtk::Orientation::Horizontal, 12);

    let icon = gtk::Image::from_icon_name("dialog-information-symbolic");
    icon.add_css_class("dim-label");
    icon.set_pixel_size(28);
    icon.set_valign(gtk::Align::Start);
    content.append(&icon);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title = gtk::Label::new(Some(&super::tr("Only part of the CSV records is loaded")));
    title.add_css_class("title-4");
    title.set_xalign(0.0);
    title.set_width_chars(1);
    title.set_wrap(true);
    title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    text_box.append(&title);

    let label = ui::selectable_wrapped_label(message);
    label.set_width_chars(1);
    label.set_max_width_chars(78);
    text_box.append(&label);
    content.append(&text_box);

    let reload_button = ui::plain_text_icon_button(
        "view-refresh-symbolic",
        "Reload All",
        "Force reload all CSV files",
    );
    reload_button.set_valign(gtk::Align::Start);
    register_loading_sensitive_widget(ui_handles, &reload_button);
    reload_button.set_action_name(Some("app.reload-all"));
    content.append(&reload_button);

    card.append(&content);
    card
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
