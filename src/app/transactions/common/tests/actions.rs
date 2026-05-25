use super::super::*;
use super::{budget, tx};

#[test]
fn transfer_transactions_show_undo_instead_of_mark_transfer() {
    let transfer = tx(-20, "TRANSFER", "Transfers");
    assert!(!transaction_is_markable_as_transfer(&transfer, &[]));

    let configured_transfer = tx(-20, "MOVE", "Internal move");
    assert!(!transaction_is_markable_as_transfer(
        &configured_transfer,
        &[budget("MOVE", BudgetDirection::Transfer)],
    ));

    let expense = tx(-20, "FOOD", "Groceries");
    assert!(transaction_is_markable_as_transfer(&expense, &[]));

    let simple_actions = visible_transaction_detail_actions(false, false, true, false);
    assert!(simple_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(!simple_actions.contains(&TransactionDetailAction::MarkTransfer));

    let advanced_actions = visible_transaction_detail_actions(true, false, true, false);
    assert!(advanced_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(!advanced_actions.contains(&TransactionDetailAction::MarkTransfer));
}

#[test]
fn simple_mode_hides_rule_and_budget_editing_transaction_actions() {
    let simple_actions = visible_transaction_detail_actions(false, true, true, false);
    assert!(!simple_actions.contains(&TransactionDetailAction::CreateRule));
    assert!(!simple_actions.contains(&TransactionDetailAction::EditBudgetCode));
    assert!(simple_actions.contains(&TransactionDetailAction::MarkTransfer));
    assert!(!simple_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(simple_actions.contains(&TransactionDetailAction::MoveBudgetCode));
    assert!(simple_actions.contains(&TransactionDetailAction::DuplicateAsFake));
    assert!(simple_actions.contains(&TransactionDetailAction::Similar));

    let advanced_actions = visible_transaction_detail_actions(true, true, true, false);
    assert!(advanced_actions.contains(&TransactionDetailAction::CreateRule));
    assert!(advanced_actions.contains(&TransactionDetailAction::EditBudgetCode));
    assert!(advanced_actions.contains(&TransactionDetailAction::MarkTransfer));
    assert!(!advanced_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(
        !visible_transaction_detail_actions(false, false, true, false)
            .contains(&TransactionDetailAction::MarkTransfer)
    );
    assert!(
        !visible_transaction_detail_actions(true, false, true, false)
            .contains(&TransactionDetailAction::MarkTransfer)
    );
}

#[test]
fn auto_detected_transactions_show_mark_invalid_action() {
    let regular_actions = visible_transaction_detail_actions(false, true, true, false);
    assert!(!regular_actions.contains(&TransactionDetailAction::MarkInvalid));

    let auto_detected_actions = visible_transaction_detail_actions(false, true, true, true);
    assert!(auto_detected_actions.contains(&TransactionDetailAction::MarkInvalid));
    let auto_detected_transfer_actions =
        visible_transaction_detail_actions(true, false, true, true);
    assert!(auto_detected_transfer_actions.contains(&TransactionDetailAction::MarkInvalid));
    assert!(auto_detected_transfer_actions.contains(&TransactionDetailAction::UndoTransfer));
}

#[test]
fn transfers_move_budget_code_action_to_menu() {
    assert_eq!(
        transaction_detail_move_budget_code_placement(false),
        TransactionDetailActionPlacement::Primary
    );
    assert_eq!(
        transaction_detail_move_budget_code_placement(true),
        TransactionDetailActionPlacement::Menu
    );
}

#[test]
fn invalid_auto_detection_rule_resets_expenses_to_other() {
    let mut transaction = tx(-20, "TRANSFER", "Transfers");
    transaction.counterparty = "Corner shop".to_string();

    let rule = invalid_auto_detection_rule_for_transaction(&transaction);

    assert_eq!(rule.priority, 150);
    assert!(rule.active);
    assert_eq!(rule.field, "counterparty");
    assert_eq!(rule.search, "Corner shop");
    assert_eq!(rule.category, tr("Other"));
    assert_eq!(rule.budget_code, "OTHER");
    assert_eq!(rule.direction, "expense");
}

#[test]
fn invalid_auto_detection_rule_resets_income_to_other_income() {
    let mut transaction = tx(20, "TRANSFER", "Transfers");
    transaction.description = "Refund".to_string();

    let rule = invalid_auto_detection_rule_for_transaction(&transaction);

    assert_eq!(rule.field, "description");
    assert_eq!(rule.search, "Refund");
    assert_eq!(rule.category, tr("Other income"));
    assert_eq!(rule.budget_code, "INC-OTHER");
    assert_eq!(rule.direction, "income");
}

#[test]
fn transfer_undo_rule_resets_expenses_to_other() {
    let mut transaction = tx(-20, "TRANSFER", "Transfers");
    transaction.counterparty = "Corner shop".to_string();

    let rule = transfer_undo_rule_for_transaction(&transaction);

    assert_eq!(rule.priority, 160);
    assert!(rule.active);
    assert_eq!(rule.field, "counterparty");
    assert_eq!(rule.search, "Corner shop");
    assert_eq!(rule.category, tr("Other"));
    assert_eq!(rule.budget_code, "OTHER");
    assert_eq!(rule.direction, "expense");
    assert_eq!(
        rule.notes,
        tr("Marked transfer undone from transaction detail.")
    );
}

#[test]
fn transfer_undo_rule_resets_income_to_other_income() {
    let mut transaction = tx(20, "TRANSFER", "Transfers");
    transaction.description = "Refund".to_string();

    let rule = transfer_undo_rule_for_transaction(&transaction);

    assert_eq!(rule.field, "description");
    assert_eq!(rule.search, "Refund");
    assert_eq!(rule.category, tr("Other income"));
    assert_eq!(rule.budget_code, "INC-OTHER");
    assert_eq!(rule.direction, "income");
}
