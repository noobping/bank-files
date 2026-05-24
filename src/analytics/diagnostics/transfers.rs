use super::amounts::{amount_cents, amount_stats};
use super::days_between;
use super::labels::{transaction_pattern_pair_match_labels, transfer_label, transfer_pair_score};
use super::transaction_key;
use crate::analytics::{TransactionPattern, TransactionPatternKind};
use crate::model::Transaction;

use std::collections::{HashMap, HashSet};

pub(super) fn transfer_transaction_patterns(
    transactions: &[Transaction],
) -> Vec<TransactionPattern> {
    let mut positives_by_amount: HashMap<i64, Vec<usize>> = HashMap::new();
    let mut expenses = Vec::new();
    for (index, transaction) in transactions.iter().enumerate() {
        let Some(cents) = amount_cents(transaction.amount) else {
            continue;
        };
        if cents > 0 {
            positives_by_amount.entry(cents).or_default().push(index);
        } else if cents < 0 {
            expenses.push(index);
        }
    }

    let mut used = HashSet::new();
    let mut patterns = Vec::new();
    for expense_index in expenses {
        if used.contains(&expense_index) {
            continue;
        }
        let expense = &transactions[expense_index];
        let Some(target) = amount_cents(-expense.amount) else {
            continue;
        };
        let Some(income_index) = positives_by_amount.get(&target).and_then(|candidates| {
            candidates
                .iter()
                .copied()
                .filter(|index| !used.contains(index))
                .filter(|index| days_between(expense.date, transactions[*index].date) <= 5)
                .filter(|index| transfer_pair_score(expense, &transactions[*index]) >= 2)
                .min_by_key(|index| days_between(expense.date, transactions[*index].date))
        }) else {
            continue;
        };

        let income = &transactions[income_index];
        let first_date = expense.date.min(income.date);
        let last_date = expense.date.max(income.date);
        patterns.push(TransactionPattern {
            kind: TransactionPatternKind::Transfer,
            label: transfer_label(expense, income),
            match_labels: transaction_pattern_pair_match_labels(expense, income),
            transaction_keys: vec![transaction_key(expense), transaction_key(income)],
            count: 2,
            amount: -expense.amount,
            amount_stats: amount_stats(&[expense_index, income_index], transactions),
            net: expense.amount + income.amount,
            first_date,
            last_date,
        });
        used.insert(expense_index);
        used.insert(income_index);
    }

    patterns.truncate(8);
    patterns
}
