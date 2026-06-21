use super::*;

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
