use super::super::super::analytics;
use super::{TransactionAmountFilter, TransactionFilter};
use crate::model::{BudgetCode, Transaction};

impl TransactionFilter {
    pub(in crate::app) fn matches(
        &self,
        tx: &crate::model::Transaction,
        budgets: &[BudgetCode],
    ) -> bool {
        match self {
            Self::All => true,
            Self::UnconfiguredBudgets => {
                analytics::transaction_has_unconfigured_expense_budget(tx, budgets)
            }
            Self::OtherCategories => matches!(tx.budget_code.trim(), "OTHER" | "INC-OTHER"),
            Self::CategoryForYear { category, year } => {
                tx.year() == *year && transaction_category_label(tx) == category.trim()
            }
            Self::Scoped {
                budget_code,
                year,
                month,
                amount,
            } => {
                if let Some(month) = month {
                    if tx.month_key() != *month {
                        return false;
                    }
                } else if let Some(year) = year {
                    if tx.year() != *year {
                        return false;
                    }
                }
                if let Some(code) = budget_code {
                    if tx.budget_code.trim() != code.trim() {
                        return false;
                    }
                }
                match amount {
                    Some(TransactionAmountFilter::Income) => {
                        !analytics::transaction_is_budget_neutral(tx, budgets)
                            && tx.amount > rust_decimal::Decimal::ZERO
                    }
                    Some(TransactionAmountFilter::Expense) => {
                        !analytics::transaction_is_budget_neutral(tx, budgets)
                            && tx.amount < rust_decimal::Decimal::ZERO
                    }
                    Some(TransactionAmountFilter::Transfer) => {
                        analytics::transaction_is_transfer(tx, budgets)
                    }
                    Some(TransactionAmountFilter::Refund) => {
                        analytics::transaction_is_refund(tx, budgets)
                    }
                    None => true,
                }
            }
        }
    }
}

fn transaction_category_label(tx: &Transaction) -> &str {
    let category = tx.category.trim();
    if category.is_empty() {
        "Uncategorized"
    } else {
        category
    }
}
