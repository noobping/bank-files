use crate::analytics::transaction_is_transfer;
use crate::model::{BudgetCode, Transaction};

use rust_decimal::Decimal;

pub fn unconfigured_expense_budget_count(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> usize {
    transactions
        .iter()
        .filter(|tx| transaction_has_unconfigured_expense_budget(tx, budgets))
        .count()
}

pub fn transaction_has_unconfigured_expense_budget(
    tx: &Transaction,
    budgets: &[BudgetCode],
) -> bool {
    if tx.amount >= Decimal::ZERO || transaction_is_transfer(tx, budgets) {
        return false;
    }

    let code = tx.budget_code.trim();
    code.is_empty()
        || !budgets.iter().any(|budget| {
            budget.direction.is_expense() && budget.code.trim().eq_ignore_ascii_case(code)
        })
}

pub fn other_category_count(transactions: &[Transaction]) -> usize {
    transactions
        .iter()
        .filter(|tx| matches!(tx.budget_code.trim(), "OTHER" | "INC-OTHER"))
        .count()
}
