use super::amounts::{amount_cents, amount_stats};
use super::days_between;
use super::labels::{
    display_label, labels_related, looks_like_refund, normalized_label,
    transaction_pattern_match_labels, transaction_pattern_pair_match_labels,
};
use super::transaction_key;
use crate::analytics::{TransactionPattern, TransactionPatternKind};
use crate::model::Transaction;

use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

pub(super) fn offsetting_transaction_patterns(
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
    expenses.sort_by_key(|index| transactions[*index].date);

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

        if let Some(refund_index) = refund_match(
            transactions,
            &positives_by_amount,
            &used,
            expense_index,
            target,
        ) {
            let refund = &transactions[refund_index];
            let first_date = expense.date.min(refund.date);
            let last_date = expense.date.max(refund.date);
            patterns.push(TransactionPattern {
                kind: TransactionPatternKind::FullRefund,
                label: display_label(expense),
                match_labels: transaction_pattern_pair_match_labels(expense, refund),
                transaction_keys: vec![transaction_key(expense), transaction_key(refund)],
                count: 2,
                amount: -expense.amount,
                amount_stats: amount_stats(&[expense_index, refund_index], transactions),
                net: expense.amount + refund.amount,
                first_date,
                last_date,
            });
            used.insert(expense_index);
            used.insert(refund_index);
            continue;
        }

        let candidates = split_candidates(transactions, &used, expense_index, target);
        if let Some(split_indexes) = subset_sum(&candidates, target) {
            let mut all_indexes = split_indexes.clone();
            all_indexes.push(expense_index);
            let first_date = all_indexes
                .iter()
                .map(|index| transactions[*index].date)
                .min()
                .unwrap_or(expense.date);
            let last_date = all_indexes
                .iter()
                .map(|index| transactions[*index].date)
                .max()
                .unwrap_or(expense.date);
            let net = all_indexes.iter().fold(Decimal::ZERO, |total, index| {
                total + transactions[*index].amount
            });
            patterns.push(TransactionPattern {
                kind: TransactionPatternKind::BillSplit,
                label: display_label(expense),
                match_labels: all_indexes
                    .iter()
                    .flat_map(|index| transaction_pattern_match_labels(&transactions[*index]))
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>(),
                transaction_keys: all_indexes
                    .iter()
                    .map(|index| transaction_key(&transactions[*index]))
                    .collect(),
                count: all_indexes.len(),
                amount: -expense.amount,
                amount_stats: amount_stats(&all_indexes, transactions),
                net,
                first_date,
                last_date,
            });
            used.insert(expense_index);
            for index in split_indexes {
                used.insert(index);
            }
        }
    }

    patterns.truncate(8);
    patterns
}

fn refund_match(
    transactions: &[Transaction],
    positives_by_amount: &HashMap<i64, Vec<usize>>,
    used: &HashSet<usize>,
    expense_index: usize,
    target: i64,
) -> Option<usize> {
    let expense = &transactions[expense_index];
    let expense_label = normalized_label(expense);
    positives_by_amount.get(&target).and_then(|candidates| {
        candidates
            .iter()
            .copied()
            .filter(|index| !used.contains(index))
            .filter(|index| days_between(expense.date, transactions[*index].date) <= 45)
            .filter(|index| {
                labels_related(&expense_label, &normalized_label(&transactions[*index]))
                    || looks_like_refund(&transactions[*index])
            })
            .min_by_key(|index| days_between(expense.date, transactions[*index].date))
    })
}

fn split_candidates(
    transactions: &[Transaction],
    used: &HashSet<usize>,
    expense_index: usize,
    target: i64,
) -> Vec<(usize, i64)> {
    let expense = &transactions[expense_index];
    let mut candidates = transactions
        .iter()
        .enumerate()
        .filter(|(index, transaction)| {
            if *index == expense_index
                || used.contains(index)
                || transaction.amount <= Decimal::ZERO
            {
                return false;
            }
            let days_after = transaction
                .date
                .signed_duration_since(expense.date)
                .num_days();
            (-2..=21).contains(&days_after)
        })
        .filter_map(|(index, transaction)| {
            amount_cents(transaction.amount)
                .filter(|amount| *amount > 0 && *amount < target)
                .map(|amount| (index, amount))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.1));
    candidates.truncate(12);
    candidates
}

fn subset_sum(candidates: &[(usize, i64)], target: i64) -> Option<Vec<usize>> {
    fn search(
        candidates: &[(usize, i64)],
        target: i64,
        start: usize,
        total: i64,
        chosen: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        if chosen.len() >= 2 && (total - target).abs() <= 1 {
            return Some(chosen.clone());
        }
        if total >= target || chosen.len() >= 6 {
            return None;
        }
        for offset in start..candidates.len() {
            chosen.push(candidates[offset].0);
            if let Some(result) = search(
                candidates,
                target,
                offset + 1,
                total + candidates[offset].1,
                chosen,
            ) {
                return Some(result);
            }
            chosen.pop();
        }
        None
    }

    search(candidates, target, 0, 0, &mut Vec::new())
}
