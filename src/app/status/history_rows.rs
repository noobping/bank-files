use super::bar::status_button;
use super::feedback::show_copy_feedback;
use super::history::StatusLogEntry;
use super::*;

#[derive(Clone)]
pub(super) struct StatusHistoryRow {
    widget: gtk::Widget,
    keywords: String,
}

pub(super) fn append_status_history_rows(
    window: &adw::ApplicationWindow,
    list: &gtk::ListBox,
    entries: &[StatusLogEntry],
) -> Vec<StatusHistoryRow> {
    let mut rows = Vec::with_capacity(entries.len());
    for entry in entries.iter().rev() {
        let row = status_history_row(window, entry);
        list.append(&row);
        rows.push(StatusHistoryRow {
            widget: row.upcast::<gtk::Widget>(),
            keywords: status_log_keywords(entry),
        });
    }
    rows
}

fn status_history_row(window: &adw::ApplicationWindow, entry: &StatusLogEntry) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.set_selectable(false);

    let container = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    container.set_margin_top(8);
    container.set_margin_bottom(8);
    container.set_margin_start(10);
    container.set_margin_end(10);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 3);
    text_box.set_hexpand(true);

    let message = ui::selectable_wrapped_label(&entry.message);
    message.set_width_chars(1);
    message.set_hexpand(true);

    let timestamp = gtk::Label::new(Some(&entry.timestamp));
    timestamp.add_css_class("caption");
    timestamp.add_css_class("dim-label");
    timestamp.set_xalign(0.0);

    text_box.append(&message);
    text_box.append(&timestamp);
    container.append(&text_box);

    let copy_button = status_button(COPY_ICON, "Copy message");
    copy_button.set_valign(gtk::Align::Center);
    let window_for_copy = window.clone();
    let message_for_copy = entry.message.clone();
    let copy_button_for_copy = copy_button.clone();
    let copy_feedback_generation = Rc::new(Cell::new(0u64));
    copy_button.connect_clicked(move |_| {
        window_for_copy.clipboard().set_text(&message_for_copy);
        show_copy_feedback(&copy_button_for_copy, &copy_feedback_generation);
    });
    container.append(&copy_button);

    row.set_child(Some(&container));
    row
}

pub(super) fn connect_status_history_search(
    search_entry: &gtk::SearchEntry,
    rows: Vec<StatusHistoryRow>,
    empty_label: gtk::Label,
) {
    search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_lowercase();
        let mut visible_count = 0;
        for row in &rows {
            let visible = status_log_matches_keywords(&row.keywords, &query);
            row.widget.set_visible(visible);
            visible_count += usize::from(visible);
        }
        empty_label.set_text(&tr(if query.is_empty() {
            "No messages yet."
        } else {
            "No messages found."
        }));
        empty_label.set_visible(visible_count == 0);
    });
}

pub(super) fn status_log_keywords(entry: &StatusLogEntry) -> String {
    format!("{} {}", entry.timestamp, entry.message).to_lowercase()
}

pub(super) fn status_log_matches_keywords(keywords: &str, query: &str) -> bool {
    query.split_whitespace().all(|term| {
        if let Some(excluded) = term.strip_prefix('!') {
            excluded.is_empty() || !keywords.contains(excluded)
        } else {
            keywords.contains(term)
        }
    })
}
