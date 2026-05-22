use super::*;

type CategoryComparisonMap = HashMap<(String, String), (Totals, Option<Totals>)>;

pub fn category_totals_for_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
    limit: usize,
) -> Vec<CategorySummary> {
    category_totals_for_period(
        financial_transactions(transactions, budgets).filter(|tx| tx.month_key() == month),
        limit,
    )
}

pub fn category_totals_for_year(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    limit: usize,
) -> Vec<CategorySummary> {
    category_totals_for_period(
        financial_transactions(transactions, budgets).filter(|tx| tx.year() == year),
        limit,
    )
}

fn category_totals_for_period<'a>(
    transactions: impl Iterator<Item = &'a Transaction>,
    limit: usize,
) -> Vec<CategorySummary> {
    let mut by_category: HashMap<(String, String), Totals> = HashMap::new();
    for tx in transactions {
        by_category
            .entry((category_label(tx).to_string(), tx.budget_code.clone()))
            .or_default()
            .add(tx);
    }

    let mut categories = by_category
        .into_iter()
        .map(|((category, budget_code), totals)| CategorySummary {
            category,
            budget_code,
            totals,
        })
        .collect::<Vec<_>>();
    categories.sort_by(|a, b| {
        b.totals
            .expenses
            .cmp(&a.totals.expenses)
            .then_with(|| a.category.cmp(&b.category))
            .then_with(|| a.budget_code.cmp(&b.budget_code))
    });
    categories.truncate(limit);
    categories
}

fn seed_expense_budget_categories(
    by_category: &mut CategoryComparisonMap,
    budgets: &[BudgetCode],
    include_previous: bool,
) {
    for budget in budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
    {
        by_category
            .entry((budget.category.clone(), budget.code.clone()))
            .or_insert_with(|| (Totals::default(), include_previous.then(Totals::default)));
    }
}

pub fn annual_budget_usage(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    comparison: ComparisonMode,
) -> Vec<AnnualBudgetUsage> {
    let include_previous = comparison.includes_previous();
    let previous_year = year - 1;
    let real_year_income =
        totals_for(financial_transactions(transactions, budgets).filter(|tx| tx.year() == year))
            .income;
    let planned_year_income = planned_year_income_total(budgets, real_year_income);
    let mut actual_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut previous_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut category_by_code: HashMap<String, String> = HashMap::new();

    for tx in transactions
        .iter()
        .filter(|tx| {
            (tx.year() == year || (include_previous && tx.year() == previous_year))
                && tx.amount < Decimal::ZERO
        })
        .filter(|tx| !transaction_is_transfer(tx, budgets))
    {
        if tx.budget_code.trim().is_empty() {
            continue;
        }
        let actuals = if tx.year() == year {
            &mut actual_by_code
        } else {
            &mut previous_by_code
        };
        *actuals.entry(tx.budget_code.clone()).or_default() += -tx.amount;
        category_by_code
            .entry(tx.budget_code.clone())
            .or_insert_with(|| tx.category.clone());
    }

    let mut rows = Vec::new();
    let mut configured_codes = HashMap::new();
    for budget in budgets
        .iter()
        .filter(|budget| budget.direction.is_expense())
    {
        configured_codes.insert(budget.code.clone(), ());
        let actual = actual_by_code
            .get(&budget.code)
            .cloned()
            .unwrap_or(Decimal::ZERO);
        let previous_actual = include_previous.then(|| {
            previous_by_code
                .get(&budget.code)
                .cloned()
                .unwrap_or(Decimal::ZERO)
        });
        let annual_budget = budget.annual_amount_with_basis(real_year_income, planned_year_income);
        rows.push(AnnualBudgetUsage {
            code: budget.code.clone(),
            category: budget.category.clone(),
            budget: annual_budget,
            actual,
            previous_actual,
            remaining: annual_budget - actual,
            budget_basis: budget.annual_budget_description(),
            notes: budget.notes.clone(),
        });
    }

    let mut unconfigured_codes = BTreeMap::new();
    for code in actual_by_code.keys() {
        if !configured_codes.contains_key(code) {
            unconfigured_codes.insert(code.clone(), ());
        }
    }
    if include_previous {
        for code in previous_by_code.keys() {
            if !configured_codes.contains_key(code) {
                unconfigured_codes.insert(code.clone(), ());
            }
        }
    }
    for code in unconfigured_codes.into_keys() {
        let actual = actual_by_code.get(&code).cloned().unwrap_or(Decimal::ZERO);
        let previous_actual = include_previous.then(|| {
            previous_by_code
                .get(&code)
                .cloned()
                .unwrap_or(Decimal::ZERO)
        });
        rows.push(AnnualBudgetUsage {
            category: category_by_code
                .get(&code)
                .cloned()
                .unwrap_or_else(|| code.clone()),
            code,
            budget: Decimal::ZERO,
            actual,
            previous_actual,
            remaining: -actual,
            budget_basis: "unconfigured budget".to_string(),
            notes: "No annual budget configured".to_string(),
        });
    }

    rows.sort_by(|a, b| a.code.cmp(&b.code));
    rows
}

pub fn category_totals_for_year_comparison(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
    limit: usize,
    comparison: ComparisonMode,
) -> Vec<YearCategoryComparison> {
    let include_previous = comparison.includes_previous();
    let previous_year = year - 1;
    let mut by_category = CategoryComparisonMap::new();
    seed_expense_budget_categories(&mut by_category, budgets, include_previous);
    for tx in transactions
        .iter()
        .filter(|tx| {
            (tx.year() == year || (include_previous && tx.year() == previous_year))
                && tx.amount < Decimal::ZERO
        })
        .filter(|tx| !transaction_is_transfer(tx, budgets))
    {
        let key = (tx.category.clone(), tx.budget_code.clone());
        let entry = by_category
            .entry(key)
            .or_insert_with(|| (Totals::default(), include_previous.then(Totals::default)));
        if tx.year() == year {
            entry.0.add(tx);
        } else if include_previous {
            entry.1.get_or_insert_with(Totals::default).add(tx);
        }
    }

    category_comparison_rows(by_category, limit)
}

fn category_comparison_rows(
    by_category: CategoryComparisonMap,
    limit: usize,
) -> Vec<YearCategoryComparison> {
    let mut categories = by_category
        .into_iter()
        .map(
            |((category, budget_code), (current, previous))| YearCategoryComparison {
                category,
                budget_code,
                current,
                previous,
            },
        )
        .collect::<Vec<_>>();
    categories.sort_by(|a, b| {
        b.current
            .expenses
            .max(previous_expenses(b))
            .cmp(&a.current.expenses.max(previous_expenses(a)))
            .then_with(|| a.category.cmp(&b.category))
    });
    categories.truncate(limit);
    categories
}

fn previous_expenses(category: &YearCategoryComparison) -> Decimal {
    category
        .previous
        .as_ref()
        .map(|totals| totals.expenses)
        .unwrap_or(Decimal::ZERO)
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

struct ActualCashFlowPeriod {
    totals: Totals,
    income: Vec<CashFlowSegment>,
    expenses: Vec<CashFlowSegment>,
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

fn category_label(tx: &Transaction) -> &str {
    let category = tx.category.trim();
    if category.is_empty() {
        "Uncategorized"
    } else {
        category
    }
}
