use super::rule_helpers::{
    editable_rule_for_transaction, invalid_auto_detection_rule_for_transaction,
    transfer_undo_rule_for_transaction,
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

pub(super) fn apply_invalid_auto_detection_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    let rule = invalid_auto_detection_rule_for_transaction(tx);
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::MarkInvalid);
}

pub(super) fn apply_transfer_undo_rule(tx: &Transaction, ui_handles: &Rc<UiHandles>) {
    let rule = transfer_undo_rule_for_transaction(tx);
    enqueue_rule_operation(ui_handles, rule, true, OperationSource::UndoTransfer);
}
