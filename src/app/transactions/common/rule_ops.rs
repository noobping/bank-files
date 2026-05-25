use super::rule_helpers::{
    editable_refund_rule_for_transaction, editable_rule_for_transaction,
    invalid_auto_detection_rule_for_transaction,
};
use super::*;

pub(super) fn apply_transaction_direction_rule(
    tx: &Transaction,
    direction: &str,
    ui_handles: &Rc<UiHandles>,
) {
    let rule = editable_rule_for_transaction(tx, Some(direction));
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::MarkTransfer);
}

pub(super) fn apply_refund_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    let rule = editable_refund_rule_for_transaction(tx);
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::MarkRefund);
}

pub(super) fn apply_invalid_auto_detection_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    let rule = invalid_auto_detection_rule_for_transaction(tx);
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::MarkInvalid);
}

pub(super) fn apply_transfer_undo_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    apply_rule_undo(tx, ui_handles, OperationSource::UndoTransfer);
}

pub(super) fn apply_refund_undo_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    apply_rule_undo(tx, ui_handles, OperationSource::UndoRefund);
}

fn apply_rule_undo(tx: &Transaction, ui_handles: &Rc<UiHandles>, source: OperationSource) {
    let Some(rule_match) = tx.rule_match.clone() else {
        show_status(ui_handles, "No matching rule was found to edit.");
        return;
    };
    enqueue_rule_undo_operation(ui_handles, rule_match, source);
}
