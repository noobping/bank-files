use super::*;

pub(in crate::app) fn transaction_matches(
    tx: &Transaction,
    budgets: &[crate::model::BudgetCode],
    filter: Option<&SearchFilter>,
) -> bool {
    filter
        .map(|filter| filter.matches_transaction(tx, budgets))
        .unwrap_or(true)
}

pub(in crate::app) fn filtered_transactions<'a>(
    transactions: &'a [Transaction],
    budgets: &[crate::model::BudgetCode],
    filter: Option<&SearchFilter>,
) -> Vec<&'a Transaction> {
    transactions
        .iter()
        .filter(|tx| transaction_matches(tx, budgets, filter))
        .collect()
}
