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

    let simple_actions = visible_transaction_detail_actions(false, false, true, true, false, true);
    assert!(simple_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(!simple_actions.contains(&TransactionDetailAction::MarkTransfer));
    assert!(!simple_actions.contains(&TransactionDetailAction::UndoRefund));

    let advanced_actions = visible_transaction_detail_actions(true, false, true, true, false, true);
    assert!(advanced_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(!advanced_actions.contains(&TransactionDetailAction::MarkTransfer));
}

#[test]
fn refunded_transactions_show_refund_undo_instead_of_mark_refund() {
    let refunded_actions =
        visible_transaction_detail_actions(false, true, false, true, false, true);

    assert!(refunded_actions.contains(&TransactionDetailAction::UndoRefund));
    assert!(!refunded_actions.contains(&TransactionDetailAction::MarkRefund));
}

#[test]
fn simple_mode_hides_rule_and_budget_editing_transaction_actions() {
    let simple_actions = visible_transaction_detail_actions(false, true, true, true, false, false);
    assert!(!simple_actions.contains(&TransactionDetailAction::CreateRule));
    assert!(!simple_actions.contains(&TransactionDetailAction::EditBudgetCode));
    assert!(simple_actions.contains(&TransactionDetailAction::MarkTransfer));
    assert!(simple_actions.contains(&TransactionDetailAction::MarkRefund));
    assert!(!simple_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(simple_actions.contains(&TransactionDetailAction::MoveBudgetCode));
    assert!(simple_actions.contains(&TransactionDetailAction::DuplicateAsFake));
    assert!(simple_actions.contains(&TransactionDetailAction::Similar));

    let advanced_actions = visible_transaction_detail_actions(true, true, true, true, false, false);
    assert!(advanced_actions.contains(&TransactionDetailAction::CreateRule));
    assert!(advanced_actions.contains(&TransactionDetailAction::EditBudgetCode));
    assert!(advanced_actions.contains(&TransactionDetailAction::MarkTransfer));
    assert!(advanced_actions.contains(&TransactionDetailAction::MarkRefund));
    assert!(!advanced_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(
        !visible_transaction_detail_actions(false, false, true, true, false, false)
            .contains(&TransactionDetailAction::MarkTransfer)
    );
    assert!(
        !visible_transaction_detail_actions(true, false, true, true, false, false)
            .contains(&TransactionDetailAction::MarkTransfer)
    );
}

#[test]
fn refund_rule_uses_refunding_and_refunded_codes_by_amount() {
    let expense = tx(-20, "FOOD", "Groceries");
    let expense_rule = editable_refund_rule_for_transaction(&expense, &[]);
    assert_eq!(expense_rule.category, tr("Refunding"));
    assert_eq!(expense_rule.budget_code, "REFUNDING");
    assert_eq!(expense_rule.direction, "expense");

    let income = tx(20, "INC-OTHER", "Other income");
    let income_rule = editable_refund_rule_for_transaction(&income, &[]);
    assert_eq!(income_rule.category, tr("Refunded"));
    assert_eq!(income_rule.budget_code, "REFUNDED");
    assert_eq!(income_rule.direction, "income");
}

#[test]
fn generated_rules_use_configured_special_budget_aliases() {
    let mut transfer_budget = budget("INTERNAL", BudgetDirection::Transfer);
    transfer_budget.special = crate::model::BudgetSpecialKind::Transfer;
    let transfer = editable_rule_for_transaction(
        &tx(-20, "OTHER", "Other"),
        Some("transfer"),
        &[transfer_budget],
    );

    assert_eq!(transfer.budget_code, "INTERNAL");

    let mut refunding_budget = budget("REFUND_OUT", BudgetDirection::Expense);
    refunding_budget.special = crate::model::BudgetSpecialKind::Refunding;
    let mut refunded_budget = budget("REFUND_IN", BudgetDirection::Income);
    refunded_budget.special = crate::model::BudgetSpecialKind::Refunded;

    let refunding = editable_refund_rule_for_transaction(
        &tx(-20, "FOOD", "Groceries"),
        &[refunding_budget.clone(), refunded_budget.clone()],
    );
    let refunded = editable_refund_rule_for_transaction(
        &tx(20, "INC-OTHER", "Other income"),
        &[refunding_budget, refunded_budget],
    );

    assert_eq!(refunding.budget_code, "REFUND_OUT");
    assert_eq!(refunded.budget_code, "REFUND_IN");
}

#[test]
fn auto_detected_transactions_show_mark_invalid_action() {
    let regular_actions = visible_transaction_detail_actions(false, true, true, true, false, false);
    assert!(!regular_actions.contains(&TransactionDetailAction::MarkInvalid));

    let auto_detected_actions =
        visible_transaction_detail_actions(false, true, true, true, true, false);
    assert!(auto_detected_actions.contains(&TransactionDetailAction::MarkInvalid));
    let auto_detected_transfer_actions =
        visible_transaction_detail_actions(true, false, true, true, true, true);
    assert!(auto_detected_transfer_actions.contains(&TransactionDetailAction::MarkInvalid));
    assert!(auto_detected_transfer_actions.contains(&TransactionDetailAction::UndoTransfer));
    assert!(!auto_detected_transfer_actions.contains(&TransactionDetailAction::UndoRefund));
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
fn transfer_undo_requires_matched_rule() {
    let without_match = visible_transaction_detail_actions(false, false, true, true, false, false);
    assert!(!without_match.contains(&TransactionDetailAction::UndoTransfer));

    let with_match = visible_transaction_detail_actions(false, false, true, true, false, true);
    assert!(with_match.contains(&TransactionDetailAction::UndoTransfer));
}

fn detail_labels(rows: &[TransactionDetailRow]) -> Vec<&'static str> {
    rows.iter().map(|row| row.label).collect()
}

#[test]
fn simple_transaction_details_hide_advanced_empty_and_duplicate_fields() {
    let mut transaction = tx(-20, "FOOD", "Groceries");
    transaction.counterparty = "Corner Shop".to_string();
    transaction.description = "corner shop".to_string();
    transaction.account = "Checking".to_string();
    transaction.transaction_id = "abc123".to_string();
    transaction.notes = "Internal note".to_string();

    let labels = detail_labels(&transaction_detail_rows(&transaction, false));

    assert!(labels.contains(&"Date"));
    assert!(labels.contains(&"Amount"));
    assert!(labels.contains(&"Counterparty"));
    assert!(labels.contains(&"Category"));
    assert!(!labels.contains(&"Description"));
    assert!(!labels.contains(&"Tags"));
    assert!(!labels.contains(&"Budget code"));
    assert!(!labels.contains(&"Account"));
    assert!(!labels.contains(&"Transaction ID"));
    assert!(!labels.contains(&"Currency"));
    assert!(!labels.contains(&"Source file"));
    assert!(!labels.contains(&"Notes"));
}

#[test]
fn advanced_transaction_details_show_advanced_non_empty_fields() {
    let mut transaction = tx(-20, "FOOD", "Groceries");
    transaction.counterparty = "Corner Shop".to_string();
    transaction.description = "Card purchase".to_string();
    transaction.account = "Checking".to_string();
    transaction.transaction_id = "abc123".to_string();
    transaction.notes = "Internal note".to_string();

    let labels = detail_labels(&transaction_detail_rows(&transaction, true));

    assert!(labels.contains(&"Description"));
    assert!(labels.contains(&"Budget code"));
    assert!(labels.contains(&"Account"));
    assert!(labels.contains(&"Transaction ID"));
    assert!(labels.contains(&"Currency"));
    assert!(labels.contains(&"Source file"));
    assert!(labels.contains(&"Notes"));
    assert!(!labels.contains(&"Tags"));
}
