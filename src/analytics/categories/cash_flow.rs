use super::*;

struct ActualCashFlowPeriod {
    totals: Totals,
    income: Vec<CashFlowSegment>,
    expenses: Vec<CashFlowSegment>,
}

pub fn cash_flow_breakdown_for_year(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    comparison: ComparisonMode,
    segment_limit: usize,
) -> Option<CashFlowBreakdown> {
    let current = cash_flow_period(transactions, budgets, year, year.to_string(), segment_limit);
    if current.totals.count == 0
        && current.planned_income.is_empty()
        && current.planned_expenses.is_empty()
    {
        return None;
    }

    let previous = comparison.includes_previous().then(|| {
        cash_flow_period(
            transactions,
            budgets,
            year - 1,
            (year - 1).to_string(),
            segment_limit,
        )
    });

    Some(CashFlowBreakdown { current, previous })
}

fn cash_flow_period(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    label: String,
    segment_limit: usize,
) -> CashFlowPeriod {
    let actual = actual_cash_flow_period(transactions, budgets, year, segment_limit);
    let real_year_income = actual.totals.income;
    let planned_year_income = planned_year_income_total(budgets, real_year_income);
    let planned_income = planned_cash_flow_segments(
        budgets,
        BudgetDirection::Income,
        real_year_income,
        planned_year_income,
        CashFlowSegmentKind::PlannedIncome,
        segment_limit,
    );
    let planned_expenses = planned_cash_flow_segments(
        budgets,
        BudgetDirection::Expense,
        real_year_income,
        planned_year_income,
        CashFlowSegmentKind::PlannedExpense,
        segment_limit,
    );

    CashFlowPeriod {
        label,
        totals: actual.totals,
        planned_income,
        actual_income: actual.income,
        planned_expenses,
        actual_expenses: actual.expenses,
    }
}

fn actual_cash_flow_period(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    segment_limit: usize,
) -> ActualCashFlowPeriod {
    let mut totals = Totals::default();
    let mut income = HashMap::<(String, String), Decimal>::new();
    let mut expenses = HashMap::<(String, String), Decimal>::new();

    for tx in financial_transactions(transactions, budgets).filter(|tx| tx.year() == year) {
        totals.add(tx);
        let key = (category_label(tx).to_string(), tx.budget_code.clone());
        if tx.amount >= Decimal::ZERO {
            *income.entry(key).or_default() += tx.amount;
        } else {
            *expenses.entry(key).or_default() += -tx.amount;
        }
    }

    ActualCashFlowPeriod {
        totals,
        income: grouped_cash_flow_segments(
            income,
            CashFlowSegmentKind::ActualIncome,
            segment_limit,
        ),
        expenses: grouped_cash_flow_segments(
            expenses,
            CashFlowSegmentKind::ActualExpense,
            segment_limit,
        ),
    }
}

fn planned_cash_flow_segments(
    budgets: &[BudgetCode],
    direction: BudgetDirection,
    real_year_income: Decimal,
    planned_year_income: Decimal,
    kind: CashFlowSegmentKind,
    limit: usize,
) -> Vec<CashFlowSegment> {
    let segments = budgets
        .iter()
        .filter(|budget| budget.direction == direction)
        .map(|budget| CashFlowSegment {
            label: budget_label(budget),
            budget_code: budget.code.clone(),
            amount: budget.annual_amount_with_basis(real_year_income, planned_year_income),
            kind,
        })
        .collect::<Vec<_>>();
    sorted_cash_flow_segments(segments, kind, limit)
}

fn grouped_cash_flow_segments(
    values: HashMap<(String, String), Decimal>,
    kind: CashFlowSegmentKind,
    limit: usize,
) -> Vec<CashFlowSegment> {
    let segments = values
        .into_iter()
        .map(|((label, budget_code), amount)| CashFlowSegment {
            label,
            budget_code,
            amount,
            kind,
        })
        .collect::<Vec<_>>();
    sorted_cash_flow_segments(segments, kind, limit)
}

fn sorted_cash_flow_segments(
    mut segments: Vec<CashFlowSegment>,
    kind: CashFlowSegmentKind,
    limit: usize,
) -> Vec<CashFlowSegment> {
    segments.retain(|segment| segment.amount > Decimal::ZERO);
    segments.sort_by(|a, b| {
        b.amount
            .cmp(&a.amount)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.budget_code.cmp(&b.budget_code))
    });
    if limit > 0 && segments.len() > limit {
        let remainder = segments
            .split_off(limit)
            .into_iter()
            .fold(Decimal::ZERO, |sum, segment| sum + segment.amount);
        if remainder > Decimal::ZERO {
            segments.push(CashFlowSegment {
                label: other_cash_flow_label(kind).to_string(),
                budget_code: String::new(),
                amount: remainder,
                kind,
            });
        }
    }
    segments
}

fn other_cash_flow_label(kind: CashFlowSegmentKind) -> &'static str {
    match kind {
        CashFlowSegmentKind::PlannedIncome => "Other planned income",
        CashFlowSegmentKind::ActualIncome => "Other income",
        CashFlowSegmentKind::PlannedExpense => "Other planned expenses",
        CashFlowSegmentKind::ActualExpense => "Other expenses",
    }
}

fn budget_label(budget: &BudgetCode) -> String {
    let category = budget.category.trim();
    if category.is_empty() {
        budget.code.clone()
    } else {
        category.to_string()
    }
}
