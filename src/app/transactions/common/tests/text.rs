use super::super::text::{transaction_subtitle, transaction_title};
use super::*;

#[test]
fn simple_transaction_subtitle_hides_budget_code() {
    let mut transaction = tx(-20, "FOOD", "Groceries");
    transaction.counterparty = "Corner Shop".to_string();
    transaction.description = "Card payment".to_string();

    assert_eq!(
        transaction_subtitle(&transaction, false),
        "2026-01-02 · Groceries · Card payment"
    );
}

#[test]
fn advanced_transaction_subtitle_includes_budget_code() {
    let mut transaction = tx(-20, "FOOD", "Groceries");
    transaction.counterparty = "Corner Shop".to_string();
    transaction.description = "Card payment".to_string();

    assert_eq!(
        transaction_subtitle(&transaction, true),
        "2026-01-02 · Groceries · FOOD · Card payment"
    );
}

#[test]
fn transaction_subtitle_hides_description_when_it_matches_title() {
    let mut transaction = tx(-20, "FOOD", "Groceries");
    transaction.counterparty = "Corner Shop".to_string();
    transaction.description = "corner shop".to_string();

    assert_eq!(transaction_title(&transaction), "Corner Shop");
    assert_eq!(
        transaction_subtitle(&transaction, false),
        "2026-01-02 · Groceries"
    );
}

#[test]
fn transaction_subtitle_hides_description_when_description_is_the_title() {
    let mut transaction = tx(-20, "", "Groceries");
    transaction.description = "Card payment".to_string();

    assert_eq!(transaction_title(&transaction), "Card payment");
    assert_eq!(
        transaction_subtitle(&transaction, false),
        "2026-01-02 · Groceries"
    );
}

#[test]
fn transaction_subtitle_keeps_tags_without_empty_separators() {
    let mut transaction = tx(-20, "", "");
    transaction.description = "Card payment".to_string();
    transaction.tags = "receipt".to_string();

    assert_eq!(
        transaction_subtitle(&transaction, false),
        "2026-01-02 · receipt"
    );
}
