use super::*;

#[test]
fn year_comparison_shows_delta_and_category_impacts() {
    let rows = vec![
        tx("2024-01-01", 200, "Income", "INC"),
        tx("2024-01-02", -50, "Groceries", "FOOD"),
        tx("2025-01-01", 250, "Income", "INC"),
        tx("2025-01-02", -80, "Groceries", "FOOD"),
        tx("2025-01-03", -20, "Transport", "TRAN"),
    ];

    let comparison = year_comparison(&rows, &[], 2025).unwrap();

    assert_eq!(comparison.previous.balance, Decimal::new(150, 0));
    assert_eq!(comparison.current.balance, Decimal::new(150, 0));
    assert_eq!(comparison.income_delta, Decimal::new(50, 0));
    assert_eq!(comparison.expense_delta, Decimal::new(50, 0));
    assert_eq!(comparison.balance_delta, Decimal::ZERO);
}

#[test]
fn dashboard_keeps_latest_two_years() {
    let mut data = AppData::default();
    data.transactions
        .push(tx("2023-12-01", 100, "Income", "INC"));
    for month in 1..=12 {
        data.transactions
            .push(tx(&format!("2024-{month:02}-01"), 100, "Income", "INC"));
    }
    for month in 1..=12 {
        data.transactions
            .push(tx(&format!("2025-{month:02}-01"), 100, "Income", "INC"));
    }

    let dashboard = dashboard(&data);

    assert_eq!(dashboard.monthly.len(), DASHBOARD_MONTH_LIMIT);
    assert_eq!(
        dashboard.monthly.first().unwrap().month,
        MonthKey::new(2024, 1)
    );
    assert_eq!(dashboard.latest_month, Some(MonthKey::new(2025, 12)));
}

#[test]
fn monthly_totals_sum_income_expenses_and_balance() {
    let rows = vec![
        tx("2025-01-01", 100, "Income", "INC"),
        tx("2025-01-02", -40, "Groceries", "FOOD"),
        tx("2025-02-01", -25, "Transport", "TRAN"),
    ];

    let months = monthly_totals_without_transfers(&rows, &[], 12);

    assert_eq!(months.len(), 2);
    assert_eq!(months[0].month, MonthKey::new(2025, 1));
    assert_eq!(months[0].totals.income, Decimal::new(100, 0));
    assert_eq!(months[0].totals.expenses, Decimal::new(40, 0));
    assert_eq!(months[0].totals.balance, Decimal::new(60, 0));
    assert_eq!(months[1].month, MonthKey::new(2025, 2));
    assert_eq!(months[1].totals.balance, Decimal::new(-25, 0));
}

#[test]
fn direct_period_totals_use_transactions_for_requested_scope() {
    let rows = vec![
        tx("2024-12-31", 999, "Income", "INC"),
        tx("2025-01-01", 100, "Income", "INC"),
        tx("2025-01-02", -40, "Groceries", "FOOD"),
        tx("2025-01-03", -200, "Savings", "TRANSFER"),
        tx("2025-01-04", -30, "Refunding", "REFUNDING"),
        tx("2025-01-05", 30, "Refunded", "REFUNDED"),
        tx("2025-02-01", -25, "Transport", "TRAN"),
    ];
    let budgets = vec![BudgetCode {
        code: "TRANSFER".to_string(),
        parent_code: String::new(),
        special: crate::model::BudgetSpecialKind::None,
        category: "Savings".to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction: BudgetDirection::Transfer,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }];

    let month = totals_for_month(&rows, &budgets, MonthKey::new(2025, 1));
    let year = totals_for_year(&rows, &budgets, 2025);

    assert_eq!(month.income, Decimal::new(100, 0));
    assert_eq!(month.expenses, Decimal::new(40, 0));
    assert_eq!(month.balance, Decimal::new(60, 0));
    assert_eq!(month.count, 2);
    assert_eq!(year.income, Decimal::new(100, 0));
    assert_eq!(year.expenses, Decimal::new(65, 0));
    assert_eq!(year.balance, Decimal::new(35, 0));
    assert_eq!(year.count, 3);
}

#[test]
fn monthly_totals_respects_limit() {
    let rows = vec![
        tx("2025-01-01", 100, "Income", "INC"),
        tx("2025-02-01", 200, "Income", "INC"),
        tx("2025-03-01", 300, "Income", "INC"),
    ];

    let months = monthly_totals_without_transfers(&rows, &[], 2);

    assert_eq!(months.len(), 2);
    assert_eq!(months[0].month, MonthKey::new(2025, 2));
    assert_eq!(months[1].month, MonthKey::new(2025, 3));
}
