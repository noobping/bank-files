use super::super::*;
use crate::analytics;
use std::collections::HashSet;

pub(super) fn generated_ignored_patterns(
    patterns: &[analytics::TransactionPattern],
) -> Vec<IgnoredTransactionPattern> {
    let mut ignored = patterns
        .iter()
        .filter(|pattern| {
            matches!(
                pattern.kind,
                analytics::TransactionPatternKind::FullRefund
                    | analytics::TransactionPatternKind::BillSplit
            )
        })
        .map(|pattern| IgnoredTransactionPattern {
            key: analytics::transaction_pattern_key(pattern),
            label: pattern.label.trim().to_string(),
        })
        .collect::<Vec<_>>();
    ignored.sort_by(|left, right| left.label.cmp(&right.label).then(left.key.cmp(&right.key)));
    ignored
}

pub(super) fn cancellation_pattern_keys(
    patterns: &[analytics::TransactionPattern],
) -> HashSet<String> {
    patterns
        .iter()
        .filter(|pattern| {
            matches!(
                pattern.kind,
                analytics::TransactionPatternKind::FullRefund
                    | analytics::TransactionPatternKind::BillSplit
            )
        })
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect()
}

pub(super) fn transfer_pattern_keys(patterns: &[analytics::TransactionPattern]) -> HashSet<String> {
    patterns
        .iter()
        .filter(|pattern| matches!(pattern.kind, analytics::TransactionPatternKind::Transfer))
        .flat_map(|pattern| pattern.transaction_keys.iter().cloned())
        .collect()
}

pub(super) fn generated_transfer_hint(transaction: &Transaction) -> bool {
    let text = normalize_key(&format!(
        "{} {} {} {} {}",
        transaction.account,
        transaction.counterparty,
        transaction.description,
        transaction.tags,
        transaction.notes
    ));
    [
        "transfer",
        "transfers",
        "internal transfer",
        "overboeking",
        "overboekingen",
        "ueberweisung",
        "uberweisung",
        "umbuchung",
    ]
    .iter()
    .any(|hint| text.contains(hint))
}

pub(super) fn transaction_key(transaction: &Transaction) -> String {
    format!(
        "{}\u{1f}{}",
        transaction.source_file, transaction.source_row
    )
}
