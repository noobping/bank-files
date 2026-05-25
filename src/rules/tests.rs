use super::defaults::AUTO_DETECTED_CATEGORY_NOTE;
use super::fallback::fallback_category;
use super::*;
use crate::model::{Transaction, TransactionRuleMatch};

use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn fallback_assigns_expenses_to_other() {
    let tx = tx("Coffee", "Cafe", -250);
    let assignment = fallback_category(&tx);

    assert_eq!(assignment.budget_code, "OTHER");
    assert_eq!(assignment.notes, None);
}

#[test]
fn fallback_assigns_income_to_other_income() {
    let tx = tx("Refund", "Shop", 250);
    let assignment = fallback_category(&tx);

    assert_eq!(assignment.budget_code, "INC-OTHER");
    assert_eq!(assignment.notes, None);
}

#[test]
fn matching_rules_override_fallbacks() {
    let mut transactions = vec![tx("Tikkie dinner", "Friend", -2300)];
    let rules = vec![Rule {
        priority: 10,
        active: true,
        field: "any".to_string(),
        pattern: "Tikkie".to_string(),
        category: "Dining out".to_string(),
        budget_code: "DINING".to_string(),
        direction: "expense".to_string(),
        amount_min: None,
        amount_max: None,
        notes: "Manual override".to_string(),
    }];

    apply_rules(&mut transactions, &rules);

    assert_eq!(transactions[0].budget_code, "DINING");
    assert_eq!(transactions[0].category, "Dining out");
    assert_eq!(
        transactions[0]
            .rule_match
            .as_ref()
            .map(|rule_match| rule_match.pattern.as_str()),
        Some("Tikkie"),
    );
    assert!(!transaction_classification_is_auto_detected(
        &transactions[0]
    ));
}

#[test]
fn fallback_clears_rule_match() {
    let mut transaction = tx("Coffee", "Cafe", -250);
    transaction.rule_match = Some(TransactionRuleMatch {
        priority: 10,
        field: "counterparty".to_string(),
        pattern: "Cafe".to_string(),
        category: "Old".to_string(),
        budget_code: "OLD".to_string(),
        direction: "expense".to_string(),
        amount_min: None,
        amount_max: None,
        notes: String::new(),
    });

    let mut transactions = vec![transaction];
    apply_rules(&mut transactions, &[]);

    assert!(transactions[0].rule_match.is_none());
}

#[test]
fn transaction_classification_indicator_follows_auto_detection_notes() {
    let mut auto = tx("Tikkie dinner", "Friend", -2300);
    auto.notes = AUTO_DETECTED_CATEGORY_NOTE.to_string();
    let mut manual = tx("Tikkie dinner", "Friend", -2300);
    manual.notes = "Manual override".to_string();

    assert!(transaction_classification_is_auto_detected(&auto));
    assert!(!transaction_classification_is_auto_detected(&manual));
}

fn tx(description: &str, counterparty: &str, cents: i64) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        amount: Decimal::new(cents, 2),
        description: description.to_string(),
        counterparty: counterparty.to_string(),
        tags: String::new(),
        account: String::new(),
        transaction_id: format!("{description}-{counterparty}-{cents}"),
        currency: "EUR".to_string(),
        category: String::new(),
        budget_code: String::new(),
        notes: String::new(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        strict_key: format!("key-{description}-{counterparty}-{cents}"),
        loose_key: format!("loose-{description}-{counterparty}-{cents}"),
        rule_match: None,
    }
}
