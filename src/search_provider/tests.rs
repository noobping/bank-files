use super::index::{
    encode_result_id, normalized_search_terms, result_id_is_valid, SearchProviderEntry,
    SearchProviderIndex,
};
use super::launch::join_search_terms;
use crate::model::{AppData, Transaction};

use rust_decimal::Decimal;
use std::str::FromStr;

fn transaction(description: &str, counterparty: &str, amount: &str) -> Transaction {
    Transaction {
        date: chrono::NaiveDate::from_ymd_opt(2026, 5, 21).expect("valid date"),
        amount: Decimal::from_str(amount).expect("valid decimal"),
        description: description.to_string(),
        counterparty: counterparty.to_string(),
        tags: String::new(),
        account: String::new(),
        transaction_id: String::new(),
        currency: "EUR".to_string(),
        source_file: "bank.csv".to_string(),
        source_row: 42,
        category: String::new(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key: format!("{description}:{counterparty}:{amount}"),
        loose_key: String::new(),
    }
}

#[test]
fn result_ids_are_opaque_hashes() {
    let tx = transaction("Groceries", "Shop", "-12.34");
    let identifier = encode_result_id(&tx);

    assert_eq!(identifier.len(), 64);
    assert!(result_id_is_valid(&identifier));
    assert!(!identifier.contains("bank.csv"));
    assert!(!identifier.contains("Groceries"));
}

#[test]
fn invalid_result_ids_are_rejected() {
    assert!(!result_id_is_valid(""));
    assert!(!result_id_is_valid("bank.csv"));
    assert!(!result_id_is_valid("xyz"));
}

#[test]
fn search_terms_join_with_spaces() {
    assert_eq!(
        join_search_terms(&["rent".to_string(), "".to_string(), "may".to_string(),]),
        "rent may".to_string()
    );
}

#[test]
fn search_matches_all_normalized_terms() {
    let tx = transaction("Monthly rent", "Housing Company", "-950");
    let entry = SearchProviderEntry::from_transaction(&tx);

    assert!(entry.matches_terms(&["rent".to_string(), "housing".to_string()]));
    assert!(!entry.matches_terms(&["salary".to_string()]));
}

#[test]
fn search_result_order_follows_transaction_order_and_limit() {
    let data = AppData {
        transactions: vec![
            transaction("Coffee", "Cafe", "-3.50"),
            transaction("Coffee beans", "Roaster", "-12.00"),
        ],
        ..AppData::default()
    };

    let index = SearchProviderIndex::from_data(&data);
    let matches = index.search_result_ids(&["coffee".to_string()], 1);

    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], encode_result_id(&data.transactions[0]));
}

#[test]
fn index_returns_metas_and_activation_queries_by_identifier() {
    let mut tx = transaction("Coffee", "Cafe", "-3.50");
    tx.transaction_id = "tx-123".to_string();
    let identifier = encode_result_id(&tx);
    let data = AppData {
        transactions: vec![tx],
        ..AppData::default()
    };
    let index = SearchProviderIndex::from_data(&data);
    let meta = index
        .meta_for_identifier(&identifier)
        .expect("known identifier should produce metadata");

    assert_eq!(
        index.activation_query_for_identifier(&identifier),
        Some("tx-123".to_string())
    );
    assert!(meta.contains_key("name"));
    assert_eq!(
        index.activation_query_for_identifier("not-a-result-id"),
        None
    );
}

#[test]
fn normalized_search_terms_drop_empty_values() {
    assert_eq!(
        normalized_search_terms(&[" Rent ".to_string(), "".to_string(), "MAY".to_string()]),
        vec!["rent".to_string(), "may".to_string()]
    );
}
