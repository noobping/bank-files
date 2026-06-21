use super::*;

#[test]
fn unconfigured_expense_budget_count_matches_transaction_filter_predicate() {
    let rows = vec![
        tx("2025-01-01", -10, "Groceries", ""),
        tx("2025-01-02", -20, "Groceries", "food"),
        tx("2025-01-03", -30, "Mystery", "UNKNOWN"),
        tx("2025-01-04", -40, "Savings", "SAVE"),
        tx("2025-01-05", 100, "Income", ""),
    ];
    let budgets = vec![
        BudgetCode {
            code: "FOOD".to_string(),
            parent_code: String::new(),
            special: crate::model::BudgetSpecialKind::None,
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "SAVE".to_string(),
            parent_code: String::new(),
            special: crate::model::BudgetSpecialKind::None,
            category: "Savings".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Transfer,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    assert_eq!(unconfigured_expense_budget_count(&rows, &budgets), 2);
    assert!(transaction_has_unconfigured_expense_budget(
        &rows[0], &budgets
    ));
    assert!(!transaction_has_unconfigured_expense_budget(
        &rows[1], &budgets
    ));
    assert!(transaction_has_unconfigured_expense_budget(
        &rows[2], &budgets
    ));
    assert!(!transaction_has_unconfigured_expense_budget(
        &rows[3], &budgets
    ));
    assert!(!transaction_has_unconfigured_expense_budget(
        &rows[4], &budgets
    ));
}
