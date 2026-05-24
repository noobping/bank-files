use super::*;
use crate::model::{DedupeMode, FieldMap, ImportReport, Transaction};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn empty_input_generates_no_user_configuration() {
    let generated = generate_automatic_configuration(&AppData::default(), true).unwrap();

    assert!(generated.summary.is_empty());
    assert!(generated.rules.is_empty());
    assert!(generated.budgets.is_empty());
    assert!(generated.ignored_patterns.is_empty());
}

#[test]
fn repeated_merchant_generates_budget_and_rule() {
    let data = app_data(complete_year_transactions(
        2025,
        -100,
        "Coffee",
        "Coffee Shop",
    ));

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert_eq!(generated.summary.complete_years, 1);
    assert_eq!(generated.summary.budget_months, 12);
    assert_eq!(generated.summary.budgets, 1);
    assert_eq!(generated.summary.rules, 1);
    assert_eq!(generated.budgets[0].category, "Coffee Shop");
    assert_eq!(generated.budgets[0].code, "COFFEE-SHOP");
    assert_eq!(generated.budgets[0].direction, "expense");
    assert_eq!(generated.budgets[0].monthly_budget, "1.00");
    assert_eq!(generated.budgets[0].yearly_budget, "12.00");
    assert_eq!(generated.rules[0].field, "counterparty");
    assert_eq!(generated.rules[0].search, "Coffee Shop");
    assert_eq!(generated.rules[0].budget_code, "COFFEE-SHOP");
}

#[test]
fn incomplete_year_transactions_do_not_generate_budget_amounts() {
    let data = app_data(vec![
        tx("2026-01-03", -500, "Coffee", "Coffee Shop", 1),
        tx("2026-01-10", -700, "Coffee", "Coffee Shop", 2),
    ]);

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert_eq!(generated.summary.complete_years, 0);
    assert_eq!(generated.summary.budget_months, 0);
    assert!(generated.budgets.is_empty());
    assert!(generated.rules.is_empty());
}

#[test]
fn complete_year_budgets_compare_years_with_average_yearly_amount() {
    let mut transactions = complete_year_transactions(2024, -100, "Coffee", "Coffee Shop");
    transactions.extend(complete_year_transactions(
        2025,
        -200,
        "Coffee",
        "Coffee Shop",
    ));
    let data = app_data(transactions);

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert_eq!(generated.summary.complete_years, 2);
    assert_eq!(generated.summary.budget_months, 24);
    assert_eq!(generated.budgets[0].monthly_budget, "1.50");
    assert_eq!(generated.budgets[0].yearly_budget, "18.00");
}

#[test]
fn detected_transfer_generates_only_transfer_configuration() {
    let data = app_data(complete_year_transfer_pairs(2025));

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert_eq!(generated.budgets.len(), 1);
    assert_eq!(generated.budgets[0].code, TRANSFER_CODE);
    assert_eq!(generated.budgets[0].direction, "transfer");
    assert_eq!(generated.rules.len(), 1);
    assert_eq!(generated.rules[0].budget_code, TRANSFER_CODE);
    assert_eq!(generated.rules[0].direction, "transfer");
    assert!(generated.ignored_patterns.is_empty());
}

#[test]
fn automatic_configuration_requires_smart_insights() {
    let data = app_data(complete_year_transfer_pairs(2025));

    let error = generate_automatic_configuration(&data, false).unwrap_err();

    assert!(format!("{error:#}").contains(SMART_INSIGHTS_REQUIRED_MESSAGE));
}

#[test]
fn refund_patterns_are_ignored_but_transfers_are_not() {
    let mut transactions = complete_year_refund_pairs(2025);
    transactions.extend(complete_year_transfer_pairs(2025));
    let data = app_data(transactions);

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert!(generated.summary.ignored_patterns > 0);
    assert!(generated
        .ignored_patterns
        .iter()
        .any(|pattern| pattern.key.starts_with("refund:")));
    assert!(!generated
        .ignored_patterns
        .iter()
        .any(|pattern| pattern.key.starts_with("transfer:")));
}

#[test]
fn detected_field_mappings_are_added_to_default_aliases() {
    let mut data = AppData::default();
    data.reports.push(ImportReport {
        rows_imported: 1,
        guessed_fields: FieldMap {
            date: Some("Booking Date Custom".to_string()),
            amount: Some("Money Column Custom".to_string()),
            ..Default::default()
        },
        ..Default::default()
    });

    let generated = generate_automatic_configuration(&data, true).unwrap();

    assert_eq!(generated.summary.field_mappings, 2);
    assert!(generated
        .aliases
        .iter()
        .any(|alias| { alias.canonical == "date" && alias.alias == "Booking Date Custom" }));
    assert!(generated
        .aliases
        .iter()
        .any(|alias| { alias.canonical == "amount" && alias.alias == "Money Column Custom" }));
}

#[test]
fn existing_transaction_category_and_code_are_not_merged() {
    let mut transactions = complete_year_transactions(2025, -100, "Coffee", "Coffee Shop");
    for transaction in &mut transactions {
        transaction.category = "Old Category".to_string();
        transaction.budget_code = "OLD".to_string();
    }

    let generated = generate_automatic_configuration(&app_data(transactions), true).unwrap();

    assert!(!generated.budgets.iter().any(|budget| budget.code == "OLD"));
    assert!(!generated.rules.iter().any(|rule| rule.budget_code == "OLD"));
}

#[test]
fn generated_budget_code_uses_readable_category_slug() {
    assert_eq!(
        generated_budget_code_for_category("Dining out & coffee", &[]),
        "DINING-OUT-COFFEE"
    );
    assert_eq!(generated_budget_code_for_category("!!!", &[]), "BUDGET");
}

#[test]
fn generated_budget_code_avoids_existing_and_reserved_codes() {
    let existing = vec!["DINING".to_string(), "DINING-2".to_string()];
    assert_eq!(
        generated_budget_code_for_category("Dining", &existing),
        "DINING-3"
    );
    assert_eq!(generated_budget_code_for_category("Inc", &[]), "INC-2");
}

fn app_data(transactions: Vec<Transaction>) -> AppData {
    AppData {
        transactions,
        dedupe_mode: DedupeMode::Disabled,
        ..Default::default()
    }
}

fn complete_year_transactions(
    year: i32,
    cents_per_month: i64,
    description: &str,
    counterparty: &str,
) -> Vec<Transaction> {
    (1..=12)
        .map(|month| {
            tx(
                &format!("{year}-{month:02}-03"),
                cents_per_month,
                description,
                counterparty,
                month as usize,
            )
        })
        .collect()
}

fn complete_year_transfer_pairs(year: i32) -> Vec<Transaction> {
    (1..=12)
        .flat_map(|month| {
            let base_row = 1_000 + month as usize * 10;
            [
                account_tx(
                    &format!("{year}-{month:02}-03"),
                    -10000,
                    "Transfer to savings",
                    "Savings",
                    "Checking",
                    base_row + 1,
                ),
                account_tx(
                    &format!("{year}-{month:02}-04"),
                    10000,
                    "Transfer from checking",
                    "Checking",
                    "Savings",
                    base_row + 2,
                ),
            ]
        })
        .collect()
}

fn complete_year_refund_pairs(year: i32) -> Vec<Transaction> {
    (1..=12)
        .flat_map(|month| {
            let base_row = 2_000 + month as usize * 10;
            [
                tx(
                    &format!("{year}-{month:02}-06"),
                    -2500,
                    "Store purchase",
                    "Store",
                    base_row + 1,
                ),
                tx(
                    &format!("{year}-{month:02}-08"),
                    2500,
                    "Refund Store",
                    "Store",
                    base_row + 2,
                ),
            ]
        })
        .collect()
}

fn tx(date: &str, cents: i64, description: &str, counterparty: &str, row: usize) -> Transaction {
    account_tx(date, cents, description, counterparty, "Checking", row)
}

fn account_tx(
    date: &str,
    cents: i64,
    description: &str,
    counterparty: &str,
    account: &str,
    row: usize,
) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount: Decimal::new(cents, 2),
        description: description.to_string(),
        counterparty: counterparty.to_string(),
        tags: String::new(),
        account: account.to_string(),
        transaction_id: format!("id-{row}"),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: row,
        category: "Uncategorized".to_string(),
        budget_code: String::new(),
        notes: String::new(),
        strict_key: format!("strict-{row}"),
        loose_key: format!("loose-{row}"),
    }
}
