use super::*;

mod controls;
mod month;
mod year;

pub(in crate::app) use month::{budget_period_row, selected_budget_month, totals_for_month};
pub(in crate::app) use year::{selected_year, year_selector_row};

fn available_budget_months(data: &AppData) -> Vec<MonthKey> {
    if data.available_months.is_empty() {
        analytics::monthly_totals_without_transfers(&data.transactions, &data.budgets, usize::MAX)
            .into_iter()
            .map(|summary| summary.month)
            .collect()
    } else {
        data.available_months.clone()
    }
}
