use super::*;

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
