use super::*;
use chrono::NaiveDate;
use rust_decimal::Decimal;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

mod config_backups;
mod config_defaults;
mod config_files;
mod rules;

#[test]
fn dedupe_enabled_removes_duplicates_and_disabled_keeps_them() {
    let transactions = vec![test_transaction("a"), test_transaction("b")];

    let (deduped, removed) = dedupe(transactions.clone(), DedupeMode::Enabled);
    assert_eq!(deduped.len(), 1);
    assert_eq!(removed, 1);

    let (kept, removed) = dedupe(transactions, DedupeMode::Disabled);
    assert_eq!(kept.len(), 2);
    assert_eq!(removed, 0);
}

fn test_transaction(id: &str) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        amount: Decimal::new(-1234, 2),
        description: "Coffee".to_string(),
        tags: String::new(),
        counterparty: "Cafe".to_string(),
        account: "NL00TEST".to_string(),
        transaction_id: id.to_string(),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        category: "Dining out".to_string(),
        budget_code: "HORECA".to_string(),
        notes: String::new(),
        strict_key: "same-strict-key".to_string(),
        loose_key: format!("loose-{id}"),
        rule_match: None,
    }
}
