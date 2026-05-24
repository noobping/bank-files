use super::defaults::{AUTO_DETECTED_CATEGORY_NOTE, GENERATED_AUTOMATIC_NOTE};
use super::fallback::fallback_category;
use super::*;
use crate::model::{BudgetCode, BudgetDirection, BudgetIncomeBasis, Transaction};

use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn fallback_recognizes_mismanagement_losses() {
    let tx = tx("Mismanagement loss belegging", "Broker Demo", -230000);
    let budgets = vec![BudgetCode {
        code: "LOSS".to_string(),
        category: "Losses & fees".to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }];
    let assignment = fallback_category(&tx, &budgets, true);
    let category = assignment.category;
    let budget_code = assignment.budget_code;

    assert!(matches!(
        category.as_str(),
        "Losses & fees" | "Verlies en kosten"
    ));
    assert_eq!(budget_code, "LOSS");
}

#[test]
fn fallback_skips_keyword_detection_when_smart_insights_disabled() {
    let tx = tx("Tikkie dinner", "Friend", -2300);
    let budgets = vec![
        BudgetCode {
            code: "TRANSFER".to_string(),
            category: "Transfers".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Transfer,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "OTHER".to_string(),
            category: "Other".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    let smart_assignment = fallback_category(&tx, &budgets, true);
    let facts_only_assignment = fallback_category(&tx, &budgets, false);

    assert_eq!(smart_assignment.budget_code, "TRANSFER");
    assert_eq!(facts_only_assignment.budget_code, "OTHER");
    assert!(smart_assignment.notes.is_some());
    assert!(facts_only_assignment.notes.is_none());
}

#[test]
fn apply_rules_skips_auto_detected_rules_when_smart_insights_disabled() {
    let mut smart_transactions = vec![tx("Tikkie dinner", "Friend", -2300)];
    let mut facts_only_transactions = smart_transactions.clone();
    let rules = vec![Rule {
        priority: 100,
        active: true,
        field: "any".to_string(),
        pattern: "Tikkie".to_string(),
        category: "Transfers".to_string(),
        budget_code: "TRANSFER".to_string(),
        direction: "transfer".to_string(),
        amount_min: None,
        amount_max: None,
        notes: GENERATED_AUTOMATIC_NOTE.to_string(),
    }];
    let budgets = vec![
        BudgetCode {
            code: "TRANSFER".to_string(),
            category: "Transfers".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Transfer,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
        BudgetCode {
            code: "OTHER".to_string(),
            category: "Other".to_string(),
            monthly_budget: None,
            yearly_budget: None,
            direction: BudgetDirection::Expense,
            income_basis: BudgetIncomeBasis::RealIncome,
            notes: String::new(),
        },
    ];

    apply_rules(&mut smart_transactions, &rules, &budgets, true);
    apply_rules(&mut facts_only_transactions, &rules, &budgets, false);

    assert_eq!(smart_transactions[0].budget_code, "TRANSFER");
    assert_eq!(facts_only_transactions[0].budget_code, "OTHER");
}

#[test]
fn manual_rules_override_auto_detected_rules_even_with_lower_priority() {
    let mut transactions = vec![tx("Tikkie dinner", "Friend", -2300)];
    let rules = vec![
        Rule {
            priority: 200,
            active: true,
            field: "any".to_string(),
            pattern: "Tikkie".to_string(),
            category: "Transfers".to_string(),
            budget_code: "TRANSFER".to_string(),
            direction: "transfer".to_string(),
            amount_min: None,
            amount_max: None,
            notes: GENERATED_AUTOMATIC_NOTE.to_string(),
        },
        Rule {
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
        },
    ];
    let budgets = vec![
        budget("TRANSFER", "Transfers", BudgetDirection::Transfer),
        budget("DINING", "Dining out", BudgetDirection::Expense),
        budget("OTHER", "Other", BudgetDirection::Expense),
    ];

    apply_rules(&mut transactions, &rules, &budgets, true);

    assert_eq!(transactions[0].budget_code, "DINING");
    assert_eq!(transactions[0].category, "Dining out");
    assert!(!transaction_classification_is_auto_detected(
        &transactions[0]
    ));
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

fn budget(code: &str, category: &str, direction: BudgetDirection) -> BudgetCode {
    BudgetCode {
        code: code.to_string(),
        category: category.to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }
}

fn tx(description: &str, counterparty: &str, cents: i64) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        amount: Decimal::new(cents, 2),
        description: description.to_string(),
        counterparty: counterparty.to_string(),
        tags: String::new(),
        account: "NL00TEST".to_string(),
        transaction_id: "test-id".to_string(),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        category: String::new(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key: "strict".to_string(),
        loose_key: "loose".to_string(),
    }
}
