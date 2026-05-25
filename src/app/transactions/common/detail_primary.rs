use super::budget_move::show_transaction_budget_code_dialog;
use super::detail_state::{
    queued_rule_undo_operation_kind, transaction_detail_config_action_blocked,
    transaction_detail_move_action_text,
};
use super::rule_ops::{apply_refund_undo_rule, apply_transfer_undo_rule};
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

pub(super) fn append_primary_transfer_undo_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    enabled: bool,
) {
    append_primary_rule_undo_action(
        tx,
        ui_handles,
        primary_actions,
        enabled,
        RuleUndoButtonSpec {
            source: OperationSource::UndoTransfer,
            label: "Undo transfer",
            tooltip: "Edit the rule by removing the matching transfer keyword",
            apply: apply_transfer_undo_rule,
        },
    );
}

pub(super) fn append_primary_refund_undo_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    enabled: bool,
) {
    append_primary_rule_undo_action(
        tx,
        ui_handles,
        primary_actions,
        enabled,
        RuleUndoButtonSpec {
            source: OperationSource::UndoRefund,
            label: "Undo refund",
            tooltip: "Edit the rule by removing the matching refund keyword",
            apply: apply_refund_undo_rule,
        },
    );
}

struct RuleUndoButtonSpec {
    source: OperationSource,
    label: &'static str,
    tooltip: &'static str,
    apply: fn(&Transaction, &Rc<UiHandles>),
}

fn append_primary_rule_undo_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    enabled: bool,
    spec: RuleUndoButtonSpec,
) {
    let tx_for_undo = tx.clone();
    let ui_for_undo = Rc::clone(ui_handles);
    let Some(rule_match) = tx.rule_match.clone() else {
        return;
    };
    let undo_operation = queued_rule_undo_operation_kind(rule_match, spec.source);
    let undo_button = ui::primary_text_icon_button("edit-undo-symbolic", spec.label, spec.tooltip);
    undo_button.set_sensitive(enabled);
    register_config_widget(ui_handles, &undo_button);
    register_operation_queue_widget(ui_handles, &undo_button, undo_operation);
    undo_button.connect_clicked(move |_| {
        if transaction_detail_config_action_blocked(
            &ui_for_undo,
            "Another edit or save is already running.",
        ) {
            return;
        }
        (spec.apply)(&tx_for_undo, &ui_for_undo);
    });
    primary_actions.append(&undo_button);
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
