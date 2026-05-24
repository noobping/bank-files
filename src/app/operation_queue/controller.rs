use super::super::*;
use super::apply::{apply_all, clear_done};
use super::messages::{operation_added_status, operation_already_queued_status};
use super::model::{EnqueueOperationResult, OperationSource};
use super::widgets::{refresh_operation_queue_ui, refresh_operation_queue_ui_for_active_session};

pub(in crate::app) fn connect_operation_queue(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let state_for_dialog = Rc::clone(state);
    let ui_for_dialog = Rc::clone(ui);
    let window_for_dialog = ui.window.clone();
    let dialog_for_button = ui.operation_queue_widgets.dialog.clone();
    ui.operation_queue_widgets.button.connect_clicked(move |_| {
        refresh_operation_queue_ui(&state_for_dialog, &ui_for_dialog);
        dialog_for_button.present(Some(&window_for_dialog));
    });

    let state_for_apply_all = Rc::clone(state);
    let ui_for_apply_all = Rc::clone(ui);
    ui.operation_queue_widgets
        .apply_all_button
        .connect_clicked(move |_| apply_all(&state_for_apply_all, &ui_for_apply_all));

    let state_for_clear_done = Rc::clone(state);
    let ui_for_clear_done = Rc::clone(ui);
    ui.operation_queue_widgets
        .clear_done_button
        .connect_clicked(move |_| clear_done(&state_for_clear_done, &ui_for_clear_done));

    let state_for_search = Rc::clone(state);
    let ui_for_search = Rc::clone(ui);
    ui.operation_queue_widgets
        .search_entry
        .connect_search_changed(move |_| {
            refresh_operation_queue_ui(&state_for_search, &ui_for_search)
        });

    refresh_operation_queue_ui(state, ui);
}

pub(in crate::app) fn enqueue_rule_operation(
    ui: &Rc<UiHandles>,
    rule: EditableRule,
    ensure_budget: bool,
    source: OperationSource,
) -> EnqueueOperationResult {
    let result = ui.operation_queue.enqueue_rule(rule, ensure_budget, source);
    refresh_operation_queue_ui_for_active_session(ui);
    if result.queued() {
        show_status(ui, operation_added_status());
    } else {
        show_status(ui, operation_already_queued_status());
    }
    result
}
