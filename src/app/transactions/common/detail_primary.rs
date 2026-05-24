use super::budget_move::show_transaction_budget_code_dialog;
use super::detail_state::{
    queued_rule_operation_kind, transaction_detail_config_action_blocked,
    transaction_detail_move_action_text,
};
use super::rule_helpers::invalid_auto_detection_rule_for_transaction;
use super::rule_ops::apply_invalid_auto_detection_rule;
use super::search::{show_transactions_text_search, similar_transaction_query};
use super::*;

pub(super) fn append_primary_move_budget_action(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    advanced_features: bool,
) {
    let tx_for_change = tx.clone();
    let state_for_change = Rc::clone(state);
    let ui_for_change = Rc::clone(ui_handles);
    let (move_label, move_tooltip) = transaction_detail_move_action_text(advanced_features);
    let move_button = ui::primary_text_icon_button("send-to-symbolic", move_label, move_tooltip);
    register_config_widget(ui_handles, &move_button);
    move_button.connect_clicked(move |_| {
        show_transaction_budget_code_dialog(&tx_for_change, &state_for_change, &ui_for_change);
    });
    primary_actions.append(&move_button);
}

pub(super) fn append_primary_invalid_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    enabled: bool,
) {
    let tx_for_invalid = tx.clone();
    let ui_for_invalid = Rc::clone(ui_handles);
    let invalid_operation = queued_rule_operation_kind(
        invalid_auto_detection_rule_for_transaction(tx),
        OperationSource::MarkInvalid,
    );
    let invalid_button = ui::primary_text_icon_button(
        "edit-undo-symbolic",
        "Mark invalid",
        "Undo this auto-detected classification",
    );
    invalid_button.set_sensitive(enabled);
    register_config_widget(ui_handles, &invalid_button);
    register_operation_queue_widget(ui_handles, &invalid_button, invalid_operation);
    invalid_button.connect_clicked(move |_| {
        if transaction_detail_config_action_blocked(
            &ui_for_invalid,
            "Another edit or save is already running.",
        ) {
            return;
        }
        apply_invalid_auto_detection_rule(&tx_for_invalid, &ui_for_invalid);
    });
    primary_actions.append(&invalid_button);
}

pub(super) fn append_similar_action(
    tx: &Transaction,
    state: &Rc<RefCell<AppData>>,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
) {
    let tx_for_similar = tx.clone();
    let state_for_similar = Rc::clone(state);
    let ui_for_similar = Rc::clone(ui_handles);
    let button =
        ui::plain_text_icon_button("edit-find-symbolic", "Similar", "Show similar transactions");
    button.connect_clicked(move |_| {
        show_transactions_text_search(
            &state_for_similar,
            &ui_for_similar,
            &similar_transaction_query(&tx_for_similar),
            "Showing similar transactions.",
        );
    });
    primary_actions.append(&button);
}
