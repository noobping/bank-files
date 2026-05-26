use super::rule_helpers::invalid_auto_detection_rule_for_transaction;
use super::*;

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
