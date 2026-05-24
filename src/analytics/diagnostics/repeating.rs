use super::amounts::{amount_stats, dominant_amount};
use super::labels::{transaction_pattern_labels, transaction_pattern_match_labels};
use super::transaction_key;
use crate::analytics::{RepeatingCadence, TransactionPattern, TransactionPatternKind};
use crate::model::Transaction;

use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

pub(super) fn repeating_transaction_patterns(
    transactions: &[Transaction],
) -> Vec<TransactionPattern> {
    let mut groups: HashMap<String, (String, Vec<usize>)> = HashMap::new();
    for (index, transaction) in transactions.iter().enumerate() {
        let mut seen = HashSet::new();
        for label in transaction_pattern_labels(transaction) {
            let key = super::labels::normalized_text(&label);
            if key.is_empty() || !seen.insert(key.clone()) {
                continue;
            }
            groups
                .entry(key)
                .or_insert_with(|| (label.trim().to_string(), Vec::new()))
                .1
                .push(index);
        }
    }

    let mut patterns = Vec::new();
    for (_, (label, mut indexes)) in groups {
        if indexes.len() < 2 {
            continue;
        }
        indexes.sort_by_key(|index| transactions[*index].date);
        let gaps = indexes
            .windows(2)
            .map(|window| {
                transactions[window[1]]
                    .date
                    .signed_duration_since(transactions[window[0]].date)
                    .num_days()
            })
            .collect::<Vec<_>>();
        let cadence = if indexes.len() == 2 {
            repeating_cadence(&gaps).filter(|cadence| *cadence == RepeatingCadence::Yearly)
        } else {
            repeating_cadence(&gaps).or_else(|| recurring_cadence(&indexes, transactions))
        };
        let Some(cadence) = cadence else {
            continue;
        };
        let first = indexes[0];
        let last = indexes[indexes.len() - 1];
        let net = indexes.iter().fold(Decimal::ZERO, |total, index| {
            total + transactions[*index].amount
        });
        let amount_stats = amount_stats(&indexes, transactions);
        let match_labels = indexes
            .iter()
            .flat_map(|index| transaction_pattern_match_labels(&transactions[*index]))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let amount = dominant_amount(&amount_stats).unwrap_or(transactions[first].amount);
        patterns.push(TransactionPattern {
            kind: TransactionPatternKind::Repeating(cadence),
            label,
            match_labels,
            transaction_keys: indexes
                .iter()
                .map(|index| transaction_key(&transactions[*index]))
                .collect(),
            count: indexes.len(),
            amount,
            amount_stats,
            net,
            first_date: transactions[first].date,
            last_date: transactions[last].date,
        });
    }

    patterns.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(right.last_date.cmp(&left.last_date))
            .then(left.label.cmp(&right.label))
    });
    patterns.truncate(8);
    patterns
}

fn repeating_cadence(gaps: &[i64]) -> Option<RepeatingCadence> {
    if gaps.is_empty() {
        return None;
    }
    let required = (gaps.len() * 2).div_ceil(3);
    [
        (RepeatingCadence::Weekly, 5_i64, 9_i64),
        (RepeatingCadence::Biweekly, 12_i64, 16_i64),
        (RepeatingCadence::Monthly, 24_i64, 38_i64),
        (RepeatingCadence::Quarterly, 70_i64, 110_i64),
        (RepeatingCadence::Yearly, 320_i64, 410_i64),
    ]
    .into_iter()
    .find(|(_, min, max)| {
        gaps.iter()
            .filter(|gap| (*min..=*max).contains(&**gap))
            .count()
            >= required.max(1)
    })
    .map(|(cadence, _, _)| cadence)
}

fn recurring_cadence(indexes: &[usize], transactions: &[Transaction]) -> Option<RepeatingCadence> {
    let months = indexes
        .iter()
        .map(|index| transactions[*index].month_key().to_string())
        .collect::<HashSet<_>>();
    (months.len() >= 3).then_some(RepeatingCadence::Recurring)
}
