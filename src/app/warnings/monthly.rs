use super::evaluator::{budget_attention_warnings, positive_budget_total};
use super::*;

pub(in crate::app) fn monthly_budget_attention_warnings(
    data: &AppData,
    month: MonthKey,
) -> Vec<AttentionWarning> {
    let real_totals = analytics::totals_for_month(&data.transactions, &data.budgets, month);
    let planned_income = analytics::planned_month_income_total(&data.budgets, real_totals.income);

    budget_attention_warnings(BudgetWarningTotals {
        real_expenses: real_totals.expenses,
        real_income: real_totals.income,
        planned_expenses: monthly_planned_expense_total(
            &data.budgets,
            real_totals.income,
            planned_income,
        ),
        planned_income,
        annual_budget_room_used: yearly_only_budget_room_used(
            &data.transactions,
            &data.budgets,
            month,
        ),
    })
}

fn monthly_planned_expense_total(
    budgets: &[BudgetCode],
    real_month_income: Decimal,
    planned_month_income: Decimal,
) -> Decimal {
    positive_budget_total(
        budgets
            .iter()
            .filter(|budget| budget.direction.is_expense())
            .filter(|budget| budget.monthly_budget.is_some())
            .map(|budget| {
                budget.monthly_amount_with_basis(real_month_income, planned_month_income)
            }),
    )
}

fn yearly_only_budget_room_used(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
    month: MonthKey,
) -> Decimal {
    let real_year_income = analytics::totals_for_year(transactions, budgets, month.year).income;
    let planned_year_income = analytics::planned_year_income_total(budgets, real_year_income);
    let mut current_actual_by_code: HashMap<String, Decimal> = HashMap::new();
    let mut earlier_actual_by_code: HashMap<String, Decimal> = HashMap::new();

    for tx in transactions
        .iter()
        .filter(|tx| tx.year() == month.year && tx.amount < Decimal::ZERO)
        .filter(|tx| !analytics::transaction_is_transfer(tx, budgets))
    {
        if tx.budget_code.trim().is_empty() {
            continue;
        }
        let tx_month = tx.month_key();
        if tx_month == month {
            *current_actual_by_code
                .entry(tx.budget_code.clone())
                .or_default() += -tx.amount;
        } else if tx_month < month {
            *earlier_actual_by_code
                .entry(tx.budget_code.clone())
                .or_default() += -tx.amount;
        }
    }

    budgets
        .iter()
        .filter(|budget| yearly_only_expense_budget(budget))
        .map(|budget| {
            let annual_budget = budget
                .annual_amount_with_basis(real_year_income, planned_year_income)
                .max(Decimal::ZERO);
            let earlier_actual = earlier_actual_by_code
                .get(&budget.code)
                .copied()
                .unwrap_or(Decimal::ZERO);
            let current_actual = current_actual_by_code
                .get(&budget.code)
                .copied()
                .unwrap_or(Decimal::ZERO);
            current_actual.min((annual_budget - earlier_actual).max(Decimal::ZERO))
        })
        .sum()
}

fn yearly_only_expense_budget(budget: &BudgetCode) -> bool {
    budget.direction.is_expense()
        && budget.monthly_budget.is_none()
        && budget.yearly_budget.is_some()
}
