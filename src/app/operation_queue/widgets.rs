use super::super::*;
use super::model::OperationQueueWidgets;
use super::presentation::{
    operation_matches_query, operation_queue_actions_are_idle, queue_summary, APPLY_ALL_TOOLTIP,
    EMPTY_QUEUE_SEARCH_TEXT, EMPTY_QUEUE_TEXT, OPERATION_QUEUE_SEARCH_PLACEHOLDER,
    OPERATION_QUEUE_TITLE,
};
use super::rows::operation_row;

pub(in crate::app) fn build_operation_queue_widgets() -> OperationQueueWidgets {
    let status_button = ui::flat_badge_icon_button("view-list-symbolic", "Processing queue");
    let button = status_button.button;
    let badge = status_button.badge;

    let shell =
        build_settings_dialog_shell(OPERATION_QUEUE_TITLE, OPERATION_QUEUE_SEARCH_PLACEHOLDER);
    let root = shell.root;
    let header = shell.header;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let apply_all_button =
        ui::primary_text_icon_button("object-select-symbolic", "Apply all", APPLY_ALL_TOOLTIP);
    header.pack_end(&apply_all_button);

    let builder = ui::builder_from_resource("operation-queue-dialog.ui");
    let content = operation_queue_object::<gtk::Box>(&builder, "operation_queue_content");
    let summary_row = operation_queue_object::<gtk::Box>(&builder, "operation_queue_summary_row");
    let summary = operation_queue_object::<gtk::Label>(&builder, "operation_queue_summary");
    let clear_done_button =
        operation_queue_object::<gtk::Button>(&builder, "operation_queue_clear_done_button");
    let list = operation_queue_object::<gtk::ListBox>(&builder, "operation_queue_list");
    root.append(&ui::action_dialog_scroll_with_min(&content, 360));

    let dialog = ui::content_dialog(tr(OPERATION_QUEUE_TITLE), &root)
        .content_width(620)
        .content_height(560)
        .build();
    ui::bind_search_bar(&dialog, &dialog, &search_bar, &search_entry);

    OperationQueueWidgets {
        button,
        badge,
        summary_row,
        summary,
        apply_all_button,
        clear_done_button,
        search_entry,
        list,
        dialog,
    }
}

fn operation_queue_object<T: IsA<gtk::glib::Object>>(builder: &gtk::Builder, id: &str) -> T {
    ui::builder_object(builder, id, "operation-queue-dialog.ui")
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
                .set_tooltip_text(Some(&tr(APPLY_ALL_TOOLTIP)));
        }
        availability => apply_action_availability(&widgets.apply_all_button, &availability),
    }
    widgets.clear_done_button.set_sensitive(applied > 0 && idle);
    widgets.clear_done_button.set_visible(applied > 0);

    ui::clear_list_box(&widgets.list);
    let operations = ui.operation_queue.operations();
    let show_summary = !operations.is_empty();
    widgets.summary_row.set_visible(show_summary);
    if show_summary {
        widgets
            .summary
            .set_text(&queue_summary(&ui.operation_queue));
    }
    if operations.is_empty() {
        widgets.list.append(&queue_text_row(&tr(EMPTY_QUEUE_TEXT)));
        refresh_menu(ui.as_ref(), &state.borrow());
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
            .append(&queue_text_row(&tr(EMPTY_QUEUE_SEARCH_TEXT)));
    }
    refresh_menu(ui.as_ref(), &state.borrow());
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
