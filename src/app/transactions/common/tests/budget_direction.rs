use super::super::*;
use super::{budget, tx};

#[test]
fn transaction_budget_direction_change_warns_for_inc_other_to_other() {
    let transaction = tx(100, "INC-OTHER", "Other income");
    assert_eq!(
        transaction_budget_direction_change(&transaction, &[], "OTHER", "Other", "expense"),
        Some(BudgetDirectionChange {
            item: "INC-OTHER -> OTHER".to_string(),
        })
    );
}

#[test]
fn transaction_budget_direction_change_warns_for_other_to_inc_other() {
    let transaction = tx(-100, "OTHER", "Other");
    assert_eq!(
        transaction_budget_direction_change(
            &transaction,
            &[],
            "INC-OTHER",
            "Other income",
            "income",
        ),
        Some(BudgetDirectionChange {
            item: "OTHER -> INC-OTHER".to_string(),
        })
    );
}

#[test]
fn transaction_budget_direction_change_uses_configured_budget_directions() {
    let transaction = tx(-100, "FOOD", "Groceries");
    assert_eq!(
        transaction_budget_direction_change(
            &transaction,
            &[
                budget("FOOD", BudgetDirection::Expense),
                budget("SALARY", BudgetDirection::Income),
            ],
            "SALARY",
            "Salary",
            "income",
        ),
        Some(BudgetDirectionChange {
            item: "FOOD -> SALARY".to_string(),
        })
    );
}

#[test]
fn transaction_budget_direction_change_ignores_same_direction_and_transfers() {
    let transaction = tx(-100, "FOOD", "Groceries");
    assert!(transaction_budget_direction_change(
        &transaction,
        &[budget("FOOD", BudgetDirection::Expense)],
        "OTHER",
        "Other",
        "expense",
    )
    .is_none());
    assert!(transaction_budget_direction_change(
        &transaction,
        &[budget("FOOD", BudgetDirection::Expense)],
        "TRANSFER",
        "Transfers",
        "transfer",
    )
    .is_none());
}
