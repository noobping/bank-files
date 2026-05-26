use super::*;

#[test]
fn annual_budget_usage_uses_year_budget_and_previous_year() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
        tx("2025-02-01", -90, "Groceries", "FOOD"),
    ];
    let budgets = vec![BudgetCode {
        code: "FOOD".to_string(),
        special: crate::model::BudgetSpecialKind::None,
        category: "Groceries".to_string(),
        monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: "Monthly budget".to_string(),
    }];

    let usage = annual_budget_usage(&rows, &budgets, 2025, ComparisonMode::WithPrevious);

    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].budget, Decimal::new(1200, 0));
    assert_eq!(usage[0].actual, Decimal::new(150, 0));
    assert_eq!(usage[0].previous_actual, Some(Decimal::new(40, 0)));
    assert_eq!(usage[0].remaining, Decimal::new(1050, 0));
}

#[test]
fn category_totals_for_year_comparison_compares_expenses() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
        tx("2025-01-02", -25, "Transport", "TRAN"),
        tx("2025-01-03", 200, "Income", "INC"),
    ];

    let categories =
        category_totals_for_year_comparison(&rows, &[], 2025, 8, ComparisonMode::WithPrevious);

    assert_eq!(categories.len(), 2);
    assert_eq!(categories[0].category, "Groceries");
    assert_eq!(categories[0].current.expenses, Decimal::new(60, 0));
    assert_eq!(
        categories[0].previous.as_ref().unwrap().expenses,
        Decimal::new(40, 0)
    );
    assert_eq!(categories[1].category, "Transport");
    assert_eq!(categories[1].current.expenses, Decimal::new(25, 0));
    assert_eq!(
        categories[1].previous.as_ref().unwrap().expenses,
        Decimal::ZERO
    );
}

#[test]
fn category_totals_for_year_uses_only_selected_year_current_totals() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
        tx("2025-01-02", 200, "Income", "INC"),
        tx("2025-01-03", -30, "Savings", "TRANSFER"),
    ];
    let budgets = vec![BudgetCode {
        code: "TRANSFER".to_string(),
        special: crate::model::BudgetSpecialKind::None,
        category: "Savings".to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction: BudgetDirection::Transfer,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }];

    let categories = category_totals_for_year(&rows, &budgets, 2025, usize::MAX);

    assert_eq!(categories.len(), 2);
    assert_eq!(categories[0].category, "Groceries");
    assert_eq!(categories[0].totals.expenses, Decimal::new(60, 0));
    assert_eq!(categories[0].totals.count, 1);
    assert_eq!(categories[1].category, "Income");
    assert_eq!(categories[1].totals.income, Decimal::new(200, 0));
    assert!(categories
        .iter()
        .all(|category| category.budget_code != "TRANSFER"));
}

#[test]
fn year_comparison_includes_configured_expense_budgets_without_transactions() {
    let rows = vec![tx("2025-01-01", -60, "Groceries", "FOOD")];
    let budgets = vec![
        BudgetCode {
            code: "FOOD".to_string(),
            special: crate::model::BudgetSpecialKind::None,
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "ENT".to_string(),
            special: crate::model::BudgetSpecialKind::None,
            category: "Subscriptions & entertainment".to_string(),
            monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(5, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    let comparison = category_totals_for_year_comparison(
        &rows,
        &budgets,
        2025,
        usize::MAX,
        ComparisonMode::WithPrevious,
    );
    let codes = comparison
        .iter()
        .map(|row| row.budget_code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&"FOOD"));
    assert!(codes.contains(&"ENT"));
    let ent = comparison
        .iter()
        .find(|row| row.budget_code == "ENT")
        .expect("ENT yearly comparison row");
    assert_eq!(ent.current.expenses, Decimal::ZERO);
    assert_eq!(ent.previous.as_ref().unwrap().expenses, Decimal::ZERO);
}

#[test]
fn annual_budget_usage_current_only_omits_previous_values() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
    ];
    let budgets = vec![BudgetCode {
        code: "FOOD".to_string(),
        special: crate::model::BudgetSpecialKind::None,
        category: "Groceries".to_string(),
        monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }];

    let usage = annual_budget_usage(&rows, &budgets, 2025, ComparisonMode::CurrentOnly);

    assert_eq!(usage[0].actual, Decimal::new(60, 0));
    assert_eq!(usage[0].previous_actual, None);
}

#[test]
fn category_totals_current_only_omits_previous_period() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
    ];

    let categories = category_totals_for_year_comparison(
        &rows,
        &[],
        2025,
        usize::MAX,
        ComparisonMode::CurrentOnly,
    );

    assert_eq!(categories.len(), 1);
    assert_eq!(categories[0].current.expenses, Decimal::new(60, 0));
    assert_eq!(categories[0].previous, None);
}
