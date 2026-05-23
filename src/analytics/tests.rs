use super::*;
use crate::model::{BudgetAmount, BudgetDirection, BudgetIncomeBasis};
use chrono::NaiveDate;

fn tx(date: &str, amount: i64, category: &str, budget_code: &str) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount: Decimal::new(amount, 0),
        description: "Test".to_string(),
        tags: String::new(),
        counterparty: "Counterparty".to_string(),
        account: "NL00TEST".to_string(),
        transaction_id: format!("{date}-{amount}"),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        category: category.to_string(),
        budget_code: budget_code.to_string(),
        notes: String::new(),
        strict_key: format!("{date}-{amount}-strict"),
        loose_key: format!("{date}-{amount}-loose"),
    }
}

#[test]
fn annual_budget_usage_uses_year_budget_and_previous_year() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
        tx("2025-02-01", -90, "Groceries", "FOOD"),
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
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "ENT".to_string(),
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
        tx("2025-02-01", -25, "Transport", "TRAN"),
    ];
    let budgets = vec![BudgetCode {
        code: "TRANSFER".to_string(),
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

#[test]
fn cash_flow_breakdown_keeps_current_and_previous_stacks_separate() {
    let rows = vec![
        tx("2024-01-03", -100, "Housing", "HOME"),
        tx("2025-01-03", -40, "Housing", "HOME"),
        tx("2024-01-24", 1000, "Income", "INC"),
        tx("2025-01-24", 1030, "Income", "INC"),
    ];

    let breakdown = cash_flow_breakdown_for_year(&rows, &[], 2025, ComparisonMode::WithPrevious, 8)
        .expect("cash-flow breakdown");
    let previous = breakdown.previous.as_ref().expect("previous stack");

    assert_eq!(breakdown.current.totals.income, Decimal::new(1030, 0));
    assert_eq!(breakdown.current.totals.expenses, Decimal::new(40, 0));
    assert_eq!(previous.totals.income, Decimal::new(1000, 0));
    assert_eq!(previous.totals.expenses, Decimal::new(100, 0));
    assert!(breakdown
        .current
        .actual_expenses
        .iter()
        .any(|segment| segment.label == "Housing" && segment.amount == Decimal::new(40, 0)));
    assert!(previous
        .actual_income
        .iter()
        .any(|segment| segment.label == "Income" && segment.amount == Decimal::new(1000, 0)));
}

#[test]
fn annual_budget_usage_current_only_omits_previous_values() {
    let rows = vec![
        tx("2024-01-01", -40, "Groceries", "FOOD"),
        tx("2025-01-01", -60, "Groceries", "FOOD"),
    ];
    let budgets = vec![BudgetCode {
        code: "FOOD".to_string(),
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

#[test]
fn cash_flow_breakdown_stacks_income_and_expense_segments() {
    let rows = vec![
        tx("2025-01-01", 100, "Salary", "INC"),
        tx("2025-01-02", 25, "Refund", "INC-OTHER"),
        tx("2025-01-03", -40, "Groceries", "FOOD"),
        tx("2025-01-04", -15, "Transport", "TRAN"),
    ];

    let breakdown =
        cash_flow_breakdown_for_year(&rows, &[], 2025, ComparisonMode::CurrentOnly, usize::MAX)
            .unwrap();

    assert_eq!(breakdown.previous, None);
    assert_eq!(breakdown.current.totals.income, Decimal::new(125, 0));
    assert_eq!(breakdown.current.totals.expenses, Decimal::new(55, 0));
    assert_eq!(breakdown.current.actual_income.len(), 2);
    assert_eq!(breakdown.current.actual_expenses.len(), 2);
}

#[test]
fn cash_flow_breakdown_adds_planned_income_and_expense_segments() {
    let rows = vec![
        tx("2025-01-01", 900, "Salary", "INC"),
        tx("2025-01-03", -80, "Groceries", "FOOD"),
    ];
    let budgets = vec![
        BudgetCode {
            code: "INC".to_string(),
            category: "Salary".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
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

    let breakdown = cash_flow_breakdown_for_year(
        &rows,
        &budgets,
        2025,
        ComparisonMode::CurrentOnly,
        usize::MAX,
    )
    .unwrap();

    assert_eq!(breakdown.current.planned_income.len(), 1);
    assert_eq!(
        breakdown.current.planned_income[0].amount,
        Decimal::new(12000, 0)
    );
    assert_eq!(breakdown.current.planned_expenses.len(), 1);
    assert_eq!(
        breakdown.current.planned_expenses[0].amount,
        Decimal::new(1200, 0)
    );
    assert_eq!(
        breakdown.current.actual_income[0].amount,
        Decimal::new(900, 0)
    );
    assert_eq!(
        breakdown.current.actual_expenses[0].amount,
        Decimal::new(80, 0)
    );
}

#[test]
fn cash_flow_planned_expense_percent_can_use_planned_income() {
    let rows = vec![tx("2025-01-01", 900, "Salary", "INC")];
    let budgets = vec![
        BudgetCode {
            code: "INC".to_string(),
            category: "Salary".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Income,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "SAVE".to_string(),
            category: "Savings".to_string(),
            monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(10, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::PlannedIncome,
            notes: String::new(),
        },
    ];

    let breakdown = cash_flow_breakdown_for_year(
        &rows,
        &budgets,
        2025,
        ComparisonMode::CurrentOnly,
        usize::MAX,
    )
    .unwrap();

    assert_eq!(
        breakdown.current.planned_expenses[0].amount,
        Decimal::new(1200, 0)
    );
}

#[test]
fn cash_flow_planned_expense_percent_uses_real_income_not_planned_income() {
    let rows = vec![tx("2025-01-01", 900, "Salary", "INC")];
    let budgets = vec![
        BudgetCode {
            code: "INC".to_string(),
            category: "Salary".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(1000, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Income,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "SAVE".to_string(),
            category: "Savings".to_string(),
            monthly_budget: Some(BudgetAmount::IncomePercent(Decimal::new(10, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    let breakdown = cash_flow_breakdown_for_year(
        &rows,
        &budgets,
        2025,
        ComparisonMode::CurrentOnly,
        usize::MAX,
    )
    .unwrap();

    assert_eq!(
        breakdown.current.planned_income[0].amount,
        Decimal::new(12000, 0)
    );
    assert_eq!(
        breakdown.current.planned_expenses[0].amount,
        Decimal::new(90, 0)
    );
}

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
            category: "Groceries".to_string(),
            monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "SAVE".to_string(),
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
