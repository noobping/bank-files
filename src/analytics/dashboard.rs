use super::*;
use chrono::Datelike;

pub fn dashboard(data: &AppData) -> Dashboard {
    let monthly =
        monthly_totals_without_transfers(&data.transactions, &data.budgets, DASHBOARD_MONTH_LIMIT);
    let latest_month = default_reporting_month(&data.transactions, &data.budgets);
    let latest_totals = latest_month
        .map(|month| totals_for_month(&data.transactions, &data.budgets, month))
        .unwrap_or_default();

    Dashboard {
        latest_month,
        latest_totals,
        all_totals: totals_for(financial_transactions(&data.transactions, &data.budgets)),
        monthly,
        top_categories: latest_month
            .map(|month| category_totals_for_month(&data.transactions, &data.budgets, month, 8))
            .unwrap_or_default(),
        budgets: latest_month
            .map(|month| budget_usage(&data.transactions, &data.budgets, month))
            .unwrap_or_default(),
    }
}

pub fn totals_for_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
) -> Totals {
    totals_for(financial_transactions(transactions, budgets).filter(|tx| tx.month_key() == month))
}

pub fn totals_for_year(transactions: &[Transaction], budgets: &[BudgetCode], year: i32) -> Totals {
    totals_for(financial_transactions(transactions, budgets).filter(|tx| tx.year() == year))
}

pub fn monthly_totals_without_transfers(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    limit: usize,
) -> Vec<MonthSummary> {
    monthly_totals_from(financial_transactions(transactions, budgets), limit)
}

pub fn default_reporting_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> Option<MonthKey> {
    let months = monthly_totals_without_transfers(transactions, budgets, usize::MAX);
    let latest = months.last().map(|summary| summary.month)?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());

    if months.iter().any(|summary| summary.month == current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

fn monthly_totals_from<'a>(
    transactions: impl Iterator<Item = &'a Transaction>,
    limit: usize,
) -> Vec<MonthSummary> {
    let mut by_month: BTreeMap<MonthKey, Totals> = BTreeMap::new();
    for tx in transactions {
        by_month.entry(tx.month_key()).or_default().add(tx);
    }

    let mut months = by_month
        .into_iter()
        .map(|(month, totals)| MonthSummary { month, totals })
        .collect::<Vec<_>>();
    if months.len() > limit {
        months = months.split_off(months.len() - limit);
    }
    months
}

pub fn year_comparison(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    year: i32,
) -> Option<YearComparison> {
    let previous_year = year - 1;
    let current = totals_for_year(transactions, budgets, year);
    if current.count == 0 {
        return None;
    }
    let previous = totals_for_year(transactions, budgets, previous_year);

    Some(YearComparison {
        year,
        previous_year,
        income_delta: current.income - previous.income,
        expense_delta: current.expenses - previous.expenses,
        balance_delta: current.balance - previous.balance,
        current,
        previous,
    })
}
