use super::labels::transaction_pattern_match_labels;
use super::offsetting::offsetting_transaction_patterns;
use super::transaction_key;
use crate::analytics::TransactionPattern;
use crate::model::Transaction;

use std::collections::HashSet;

pub fn transactions_without_canceled_patterns(transactions: &[Transaction]) -> Vec<Transaction> {
    let hidden_transaction_keys = hidden_canceled_transaction_keys(transactions);
    if hidden_transaction_keys.is_empty() {
        return transactions.to_vec();
    }

    transactions
        .iter()
        .filter(|transaction| !hidden_transaction_keys.contains(&transaction_key(transaction)))
        .cloned()
        .collect()
}

fn hidden_canceled_transaction_keys(transactions: &[Transaction]) -> HashSet<String> {
    offsetting_transaction_patterns(transactions)
        .into_iter()
        .flat_map(|pattern| pattern.transaction_keys)
        .collect()
}

pub fn transaction_matches_pattern(
    transaction: &Transaction,
    pattern: &TransactionPattern,
) -> bool {
    if !pattern.transaction_keys.is_empty() {
        return pattern
            .transaction_keys
            .iter()
            .any(|key| key == &transaction_key(transaction));
    }
    if transaction.date < pattern.first_date || transaction.date > pattern.last_date {
        return false;
    }
    let amount_matches = pattern
        .amount_stats
        .iter()
        .any(|stat| stat.amount == transaction.amount || stat.amount == -transaction.amount);
    if !amount_matches {
        return false;
    }
    let labels = transaction_pattern_match_labels(transaction);
    pattern
        .match_labels
        .iter()
        .any(|label| labels.iter().any(|candidate| candidate == label))
}
