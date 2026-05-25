use super::*;

#[test]
fn budget_usage_compares_latest_expenses_to_budget() {
    let rows = vec![
        tx("2025-03-01", -60, "Groceries", "FOOD"),
        tx("2025-03-02", -15, "Groceries", "FOOD"),
        tx("2025-03-03", 200, "Income", "INC"),
    ];
    let budgets = vec![BudgetCode {
        code: "FOOD".to_string(),
        category: "Groceries".to_string(),
        monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Monthly budget".to_string(),
    }];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].actual, Decimal::new(75, 0));
    assert_eq!(usage[0].remaining, Decimal::new(25, 0));
}

#[test]
fn budget_usage_ignores_refunding_and_refunded_budget_codes() {
    let rows = vec![
        tx("2025-03-01", -60, "Groceries", "FOOD"),
        tx("2025-03-02", -60, "Refunding", "REFUNDING"),
        tx("2025-03-03", 60, "Refunded", "REFUNDED"),
    ];
    let budgets = vec![
        BudgetCode {
            code: "FOOD".to_string(),
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "REFUNDING".to_string(),
            category: "Refunding".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].code, "FOOD");
    assert_eq!(usage[0].actual, Decimal::new(60, 0));
}

#[test]
fn dynamic_budget_usage_uses_month_income_percent() {
    let rows = vec![
        tx("2025-03-01", 1000, "Income", "INC"),
        tx("2025-03-02", -60, "Dining", "DINING"),
    ];
    let budgets = vec![BudgetCode {
        code: "DINING".to_string(),
        category: "Dining".to_string(),
        monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(10, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Income based".to_string(),
    }];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    assert_eq!(usage[0].budget, Decimal::new(100, 0));
    assert_eq!(usage[0].remaining, Decimal::new(40, 0));
    assert_eq!(usage[0].budget_basis, "10% of real income");
}

#[test]
fn dynamic_budget_usage_can_use_planned_income_percent() {
    let rows = vec![
        tx("2025-03-01", 800, "Income", "INC"),
        tx("2025-03-02", -60, "Dining", "DINING"),
    ];
    let budgets = vec![
        BudgetCode {
            code: "INC".to_string(),
            category: "Income".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Income,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "DINING".to_string(),
            category: "Dining".to_string(),
            monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(10, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::PlannedIncome,
            notes: "Income based".to_string(),
        },
    ];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    let dining = usage.iter().find(|row| row.code == "DINING").unwrap();
    assert_eq!(dining.budget, Decimal::new(100, 0));
    assert_eq!(dining.remaining, Decimal::new(40, 0));
    assert_eq!(dining.budget_basis, "10% of planned income");
}

#[test]
fn yearly_only_budget_usage_allows_one_month_to_use_the_annual_room() {
    let rows = vec![tx("2025-03-02", -1000, "Travel", "TRAVEL")];
    let budgets = vec![BudgetCode {
        code: "TRAVEL".to_string(),
        category: "Travel".to_string(),
        monthly_budget: None,
        yearly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Year pot".to_string(),
    }];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    assert_eq!(usage[0].budget, Decimal::new(1000, 0));
    assert_eq!(usage[0].actual, Decimal::new(1000, 0));
    assert_eq!(usage[0].remaining, Decimal::ZERO);
    assert_eq!(usage[0].budget_basis, "remaining yearly budget");
}

#[test]
fn yearly_only_budget_usage_uses_remaining_annual_room() {
    let rows = vec![
        tx("2025-01-02", -200, "Travel", "TRAVEL"),
        tx("2025-03-02", -900, "Travel", "TRAVEL"),
    ];
    let budgets = vec![BudgetCode {
        code: "TRAVEL".to_string(),
        category: "Travel".to_string(),
        monthly_budget: None,
        yearly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Year pot".to_string(),
    }];

    let march = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));
    let april = budget_usage(&rows, &budgets, MonthKey::new(2025, 4));

    assert_eq!(march[0].budget, Decimal::new(800, 0));
    assert_eq!(march[0].actual, Decimal::new(900, 0));
    assert_eq!(march[0].remaining, Decimal::new(-100, 0));
    assert_eq!(april[0].budget, Decimal::ZERO);
    assert_eq!(april[0].actual, Decimal::ZERO);
    assert_eq!(april[0].remaining, Decimal::ZERO);
}

#[test]
fn monthly_budget_usage_takes_precedence_over_yearly_room() {
    let rows = vec![tx("2025-01-02", -900, "Travel", "TRAVEL")];
    let budgets = vec![BudgetCode {
        code: "TRAVEL".to_string(),
        category: "Travel".to_string(),
        monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
        yearly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Monthly wins".to_string(),
    }];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 3));

    assert_eq!(usage[0].budget, Decimal::new(100, 0));
    assert_eq!(usage[0].actual, Decimal::ZERO);
    assert_eq!(usage[0].remaining, Decimal::new(100, 0));
    assert_eq!(usage[0].budget_basis, "fixed budget");
}

#[test]
fn dynamic_annual_budget_uses_year_income_percent() {
    let rows = vec![
        tx("2025-01-01", 1000, "Income", "INC"),
        tx("2025-02-01", 2000, "Income", "INC"),
        tx("2025-03-01", -250, "Savings", "SAVE"),
    ];
    let budgets = vec![BudgetCode {
        code: "SAVE".to_string(),
        category: "Savings".to_string(),
        monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(10, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Income based".to_string(),
    }];

    let usage = annual_budget_usage(&rows, &budgets, 2025, ComparisonMode::WithPrevious);

    assert_eq!(usage[0].budget, Decimal::new(300, 0));
    assert_eq!(usage[0].remaining, Decimal::new(50, 0));
}

#[test]
fn budget_usage_includes_unconfigured_loss_codes() {
    let rows = vec![tx("2025-03-01", -2300, "Losses and fees", "LOSS")];
    let usage = budget_usage(&rows, &[], MonthKey::new(2025, 3));

    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].code, "LOSS");
    assert_eq!(usage[0].category, "Losses and fees");
    assert_eq!(usage[0].budget, Decimal::ZERO);
    assert_eq!(usage[0].actual, Decimal::new(2300, 0));
    assert_eq!(usage[0].remaining, Decimal::new(-2300, 0));
}

#[test]
fn budget_usage_ignores_income_budget_codes() {
    let rows = vec![
        tx("2025-01-02", 1200, "Income", "INC"),
        tx("2025-01-03", -40, "Groceries", "FOOD"),
    ];
    let budgets = vec![
        BudgetCode {
            code: "INC".to_string(),
            category: "Income".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(1200, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Income,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "FOOD".to_string(),
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    let usage = budget_usage(&rows, &budgets, MonthKey::new(2025, 1));

    assert!(usage.iter().all(|row| row.code != "INC"));
    assert!(usage.iter().any(|row| row.code == "FOOD"));
}
