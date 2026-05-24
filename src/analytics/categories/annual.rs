use super::*;

type CategoryComparisonMap = HashMap<(String, String), (Totals, Option<Totals>)>;

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
