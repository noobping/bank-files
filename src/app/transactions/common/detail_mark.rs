use super::detail_state::{
    append_transaction_detail_menu_action, queued_rule_operation_kind,
    transaction_detail_config_action_blocked,
};
use super::rule_helpers::{editable_refund_rule_for_transaction, editable_rule_for_transaction};
use super::rule_ops::{apply_refund_rule, apply_transaction_direction_rule};
use super::*;

pub(super) fn append_mark_transfer_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    menu: &gtk::gio::Menu,
    menu_actions: &gtk::gio::SimpleActionGroup,
    enabled: bool,
) {
    let tx_for_transfer = tx.clone();
    let ui_for_transfer = Rc::clone(ui_handles);
    let operation = queued_rule_operation_kind(
        editable_rule_for_transaction(tx, Some("transfer")),
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
            apply_transaction_direction_rule(&tx_for_transfer, "transfer", &ui_for_transfer);
        },
    );
    register_operation_queue_menu_action(ui_handles, primary_actions, &action, operation);
}

pub(super) fn append_mark_refund_action(
    tx: &Transaction,
    ui_handles: &Rc<UiHandles>,
    primary_actions: &gtk::Box,
    menu: &gtk::gio::Menu,
    menu_actions: &gtk::gio::SimpleActionGroup,
    enabled: bool,
) {
    let tx_for_refund = tx.clone();
    let ui_for_refund = Rc::clone(ui_handles);
    let operation = queued_rule_operation_kind(
        editable_refund_rule_for_transaction(tx),
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
            apply_refund_rule(&tx_for_refund, &ui_for_refund);
        },
    );
    register_operation_queue_menu_action(ui_handles, primary_actions, &action, operation);
}
