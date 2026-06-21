use super::super::*;
use crate::model::BudgetCode;

pub(super) fn sort_transactions(transactions: &mut [Transaction]) {
    transactions.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| left.description.cmp(&right.description))
    });
}

pub(super) fn months_from_transactions(transactions: &[Transaction]) -> Vec<MonthKey> {
    transactions
        .iter()
        .map(Transaction::month_key)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn years_from_months(months: &[MonthKey]) -> Vec<i32> {
    months
        .iter()
        .map(|month| month.year)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn default_month_from_available_months(months: &[MonthKey]) -> Option<MonthKey> {
    use chrono::Datelike;

    let latest = months.last().copied()?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());
    if months.contains(&current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

pub(super) fn analytics_default_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> Option<MonthKey> {
    crate::analytics::default_reporting_month(transactions, budgets)
}
