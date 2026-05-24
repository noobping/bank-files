use super::amounts::dominant_amount;
use super::labels::normalized_text;
use super::transaction_pattern_kind_key;
use crate::analytics::{TransactionPattern, TransactionPatternAmountStat};

use std::collections::HashMap;

pub(super) fn group_transaction_patterns(
    patterns: Vec<TransactionPattern>,
) -> Vec<TransactionPattern> {
    let mut grouped: HashMap<(String, String), TransactionPattern> = HashMap::new();
    for pattern in patterns {
        let key = (
            transaction_pattern_kind_key(pattern.kind).to_string(),
            normalized_text(&pattern.label),
        );
        grouped
            .entry(key)
            .and_modify(|existing| {
                existing.count += pattern.count;
                existing.net += pattern.net;
                existing.first_date = existing.first_date.min(pattern.first_date);
                existing.last_date = existing.last_date.max(pattern.last_date);
                merge_amount_stats(&mut existing.amount_stats, &pattern.amount_stats);
                merge_match_labels(&mut existing.match_labels, &pattern.match_labels);
                merge_match_labels(&mut existing.transaction_keys, &pattern.transaction_keys);
                if let Some(amount) = dominant_amount(&existing.amount_stats) {
                    existing.amount = amount;
                }
            })
            .or_insert(pattern);
    }
    let mut patterns = grouped.into_values().collect::<Vec<_>>();
    for pattern in &mut patterns {
        pattern
            .amount_stats
            .sort_by_key(|stat| std::cmp::Reverse(stat.count));
    }
    patterns
}

fn merge_amount_stats(
    existing: &mut Vec<TransactionPatternAmountStat>,
    incoming: &[TransactionPatternAmountStat],
) {
    for stat in incoming {
        if let Some(existing_stat) = existing.iter_mut().find(|item| item.amount == stat.amount) {
            existing_stat.count += stat.count;
        } else {
            existing.push(stat.clone());
        }
    }
}

pub(super) fn merge_match_labels(existing: &mut Vec<String>, incoming: &[String]) {
    for label in incoming {
        if !existing.iter().any(|item| item == label) {
            existing.push(label.clone());
        }
    }
}
