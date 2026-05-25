use crate::model::{AppData, BudgetCode, BudgetDirection, ComparisonMode, MonthKey, Transaction};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap};

mod budgets;
mod categories;
mod dashboard;
mod diagnostics;
#[cfg(test)]
mod tests;
mod types;

pub use budgets::budget_usage;
pub use categories::{
    annual_budget_usage, cash_flow_breakdown_for_year, category_totals_for_month,
    category_totals_for_year, category_totals_for_year_comparison,
};
pub use dashboard::{
    dashboard, default_reporting_month, monthly_totals_without_transfers, totals_for_month,
    totals_for_year, year_comparison,
};
pub use diagnostics::{
    other_category_count, transaction_has_unconfigured_expense_budget,
    unconfigured_expense_budget_count,
};
pub use types::*;

use budgets::totals_for;

pub fn planned_month_income_total(budgets: &[BudgetCode], real_month_income: Decimal) -> Decimal {
    budgets
        .iter()
        .filter(|budget| budget.direction == BudgetDirection::Income)
        .map(|budget| budget.monthly_amount_with_basis(real_month_income, real_month_income))
        .sum()
}

pub fn planned_year_income_total(budgets: &[BudgetCode], real_year_income: Decimal) -> Decimal {
    budgets
        .iter()
        .filter(|budget| budget.direction == BudgetDirection::Income)
        .map(|budget| budget.annual_amount_with_basis(real_year_income, real_year_income))
        .sum()
}

pub fn transaction_is_transfer(tx: &Transaction, budgets: &[BudgetCode]) -> bool {
    let code = tx.budget_code.trim();
    if code.eq_ignore_ascii_case("TRANSFER") {
        return true;
    }
    !code.is_empty()
        && budgets.iter().any(|budget| {
            budget.direction.is_transfer() && budget.code.trim().eq_ignore_ascii_case(code)
        })
}

pub fn financial_transactions<'a>(
    transactions: &'a [Transaction],
    budgets: &'a [BudgetCode],
) -> impl Iterator<Item = &'a Transaction> {
    transactions
        .iter()
        .filter(move |tx| !transaction_is_transfer(tx, budgets))
}
