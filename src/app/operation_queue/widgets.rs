use super::super::*;
use super::model::OperationQueueWidgets;
use super::presentation::{
    operation_matches_query, operation_queue_actions_are_idle, queue_summary,
};
use super::rows::operation_row;

pub(in crate::app) fn build_operation_queue_widgets() -> OperationQueueWidgets {
    let badge = gtk::Label::new(None);
    badge.add_css_class("caption");
    badge.set_visible(false);

    let icon = gtk::Image::from_icon_name("view-list-symbolic");
    let icon_shell = gtk::Overlay::new();
    icon_shell.set_child(Some(&icon));

    let button_content = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    badge.set_halign(gtk::Align::Center);
    badge.set_valign(gtk::Align::Center);
    button_content.append(&badge);
    button_content.append(&icon_shell);

    let button = ui::flat_custom_button("Processing queue", &button_content);
    button.set_focus_on_click(false);

    let shell = build_settings_dialog_shell("Processing Queue", "Search queued operations");
    let root = shell.root;
    let header = shell.header;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let apply_all_button = ui::primary_text_icon_button(
        "object-select-symbolic",
        "Apply all",
        "Apply all pending queued operations",
    );
    let clear_done_button = ui::icon_button("edit-clear-symbolic", "Clear completed operations");
    clear_done_button.add_css_class("flat");
    header.pack_end(&apply_all_button);

    let content = ui::page_box();
    let summary = gtk::Label::new(None);
    summary.add_css_class("dim-label");
    summary.set_selectable(false);
    summary.set_xalign(0.0);
    summary.set_width_chars(1);
    summary.set_wrap(true);
    summary.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    let summary_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    summary_row.set_hexpand(true);
    summary.set_hexpand(true);
    summary_row.append(&summary);
    summary_row.append(&clear_done_button);
    content.append(&summary_row);

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    list.set_selection_mode(gtk::SelectionMode::None);
    list.set_hexpand(true);
    content.append(&list);
    root.append(&ui::action_dialog_scroll_with_min(&content, 360));

    let dialog = ui::content_dialog(tr("Processing Queue"), &root)
        .content_width(620)
        .content_height(560)
        .build();
    ui::connect_search_shortcut(&dialog, &search_bar, &search_entry);
    search_bar.set_key_capture_widget(Some(&dialog));

    OperationQueueWidgets {
        button,
        badge,
        summary,
        apply_all_button,
        clear_done_button,
        search_entry,
        list,
        dialog,
    }
}

pub(super) fn refresh_operation_queue_ui_for_active_session(ui: &Rc<UiHandles>) {
    ACTIVE_SESSION.with(|active| {
        if let Some(session) = active.borrow().clone() {
            if Rc::ptr_eq(&session.ui, ui) {
                refresh_operation_queue_ui(&session.state, &session.ui);
            }
        }
    });
}

pub(in crate::app) fn refresh_active_operation_queue_ui() {
    ACTIVE_SESSION.with(|active| {
        if let Some(session) = active.borrow().clone() {
            refresh_operation_queue_ui(&session.state, &session.ui);
        }
    });
}

pub(super) fn refresh_operation_queue_ui(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    update_operation_queue_action_widgets(ui.as_ref());
    let widgets = &ui.operation_queue_widgets;
    let actionable = ui.operation_queue.actionable_count();
    let applied = ui.operation_queue.applied_count();
    widgets.badge.set_visible(actionable > 0);
    widgets.badge.set_text(&actionable.to_string());
    set_operation_queue_button_style(&widgets.button, actionable);
    widgets.button.set_tooltip_text(Some(&trf(
        "Processing queue: {count} pending",
        &[("count", actionable.to_string())],
    )));
    let idle = operation_queue_actions_are_idle(
        ui.operation_queue.is_processing(),
        ui.loading_count.get(),
    );
    match config_write_availability(ui.as_ref()) {
        ActionAvailability::Available => {
            widgets.apply_all_button.set_visible(true);
            widgets
                .apply_all_button
                .set_sensitive(actionable > 0 && idle);
            widgets
                .apply_all_button
                .set_tooltip_text(Some(&tr("Apply all pending queued operations")));
        }
        availability => apply_action_availability(&widgets.apply_all_button, &availability),
    }
    widgets.clear_done_button.set_sensitive(applied > 0 && idle);
    widgets.clear_done_button.set_visible(applied > 0);
    widgets
        .summary
        .set_text(&queue_summary(&ui.operation_queue));

    ui::clear_list_box(&widgets.list);
    let operations = ui.operation_queue.operations();
    if operations.is_empty() {
        widgets
            .list
            .append(&queue_text_row(&tr("No queued operations.")));
        return;
    }

    let query = widgets.search_entry.text().trim().to_lowercase();
    let mut visible_count = 0;
    for operation in operations {
        if operation_matches_query(&operation, &query) {
            visible_count += 1;
            widgets.list.append(&operation_row(state, ui, operation));
        }
    }

    if visible_count == 0 {
        widgets
            .list
            .append(&queue_text_row(&tr("No queued operations found.")));
    }
}

fn set_operation_queue_button_style(button: &gtk::Button, actionable: usize) {
    if operation_queue_button_is_suggested(actionable) {
        button.remove_css_class("flat");
        button.add_css_class("suggested-action");
    } else {
        button.remove_css_class("suggested-action");
        button.add_css_class("flat");
    }
}

pub(super) fn operation_queue_button_is_suggested(actionable: usize) -> bool {
    actionable > 0
}

fn queue_text_row(text: &str) -> adw::ActionRow {
    ui::text_list_row(text)
}
