use super::detail_state::{
    append_transaction_detail_menu_action, queued_rule_operation_kind,
    transaction_detail_config_action_blocked,
};
use super::rule_helpers::{editable_refund_rule_for_transaction, editable_rule_for_transaction};
use super::*;
use crate::model::BudgetCode;

pub(super) fn append_mark_transfer_action(
    tx: &Transaction,
    budgets: &[BudgetCode],
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    menu: &gtk::gio::Menu,
    menu_actions: &gtk::gio::SimpleActionGroup,
    enabled: bool,
) {
    let rule_for_transfer = editable_rule_for_transaction(tx, Some("transfer"), budgets);
    let ui_for_transfer = Rc::clone(ui_handles);
    let operation = queued_rule_operation_kind(
        editable_rule_for_transaction(tx, Some("transfer"), budgets),
        OperationSource::MarkTransfer,
    );
    let action = append_transaction_detail_menu_action(
        menu,
        menu_actions,
        "mark-transfer",
        "Mark transfer",
        enabled,
        move || {
            if transaction_detail_config_action_blocked(
                &ui_for_transfer,
                "Another edit or save is already running.",
            ) {
                return;
            }
            enqueue_rule_operation(
                &ui_for_transfer,
                rule_for_transfer.clone(),
                true,
                OperationSource::MarkTransfer,
            );
        },
    );
    register_operation_queue_menu_action(ui_handles, primary_actions, &action, operation);
}

pub(super) fn append_mark_refund_action(
    tx: &Transaction,
    budgets: &[BudgetCode],
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    menu: &gtk::gio::Menu,
    menu_actions: &gtk::gio::SimpleActionGroup,
    enabled: bool,
) {
    let rule_for_refund = editable_refund_rule_for_transaction(tx, budgets);
    let ui_for_refund = Rc::clone(ui_handles);
    let operation = queued_rule_operation_kind(
        editable_refund_rule_for_transaction(tx, budgets),
        OperationSource::MarkRefund,
    );
    let action = append_transaction_detail_menu_action(
        menu,
        menu_actions,
        "mark-refund",
        "Mark refund",
        enabled,
        move || {
            if transaction_detail_config_action_blocked(
                &ui_for_refund,
                "Another edit or save is already running.",
            ) {
                return;
            }
            enqueue_rule_operation(
                &ui_for_refund,
                rule_for_refund.clone(),
                true,
                OperationSource::MarkRefund,
            );
        },
    );
    register_operation_queue_menu_action(ui_handles, primary_actions, &action, operation);
}
