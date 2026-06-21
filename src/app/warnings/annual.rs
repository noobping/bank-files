use super::evaluator::{budget_attention_warnings, positive_budget_total};
use super::*;

pub(in crate::app) fn annual_budget_attention_warnings(
    data: &AppData,
    year: i32,
) -> Vec<AttentionWarning> {
    let real_totals = analytics::totals_for_year(&data.transactions, &data.budgets, year);
    let planned_income = analytics::planned_year_income_total(&data.budgets, real_totals.income);

    budget_attention_warnings(BudgetWarningTotals {
        real_expenses: real_totals.expenses,
        real_income: real_totals.income,
        planned_expenses: positive_budget_total(
            data.budgets
                .iter()
                .filter(|budget| budget.direction.is_expense())
                .map(|budget| budget.annual_amount_with_basis(real_totals.income, planned_income)),
        ),
        planned_income,
        annual_budget_room_used: Decimal::ZERO,
    })
}
