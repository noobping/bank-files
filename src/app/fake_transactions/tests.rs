use super::super::{AppData, BudgetDirection, MonthKey, Transaction};
use super::model::{
    data_with_fake_transactions, real_transactions, transaction_is_fake, FakeTransaction,
    FakeTransactionStore,
};
use super::presentation::{fake_transaction_matches_search, fake_transaction_search_terms};
use super::transaction_builder::{
    fake_transaction_amount_for_budget, fake_transaction_budget_code_for_save,
    normalize_fake_transaction,
};
use super::DEFAULT_FAKE_ACCOUNT;
use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::*;

fn tx(date: &str, amount: i64, description: &str) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount: Decimal::new(amount, 0),
        description: description.to_string(),
        counterparty: String::new(),
        tags: String::new(),
        account: DEFAULT_FAKE_ACCOUNT.to_string(),
        transaction_id: String::new(),
        currency: DEFAULT_FAKE_CURRENCY.to_string(),
        source_file: String::new(),
        source_row: 0,
        category: "Other".to_string(),
        budget_code: "OTHER".to_string(),
        notes: String::new(),
        strict_key: String::new(),
        loose_key: String::new(),
    }
}

fn budget(code: &str, category: &str, direction: BudgetDirection) -> crate::model::BudgetCode {
    crate::model::BudgetCode {
        code: code.to_string(),
        category: category.to_string(),
        monthly_budget: None,
        yearly_budget: None,
        direction,
        income_basis: crate::model::BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }
}

#[test]
fn fake_transaction_amount_uses_budget_code_direction() {
    let budgets = vec![
        budget("FOOD", "Food", BudgetDirection::Expense),
        budget("SALARY", "Salary", BudgetDirection::Income),
    ];

    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(25, 0), "FOOD", "Food", &budgets),
        Decimal::new(-25, 0)
    );
    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(-25, 0), "SALARY", "Salary", &budgets),
        Decimal::new(25, 0)
    );
}

#[test]
fn fake_transaction_amount_keeps_transfer_sign() {
    let budgets = vec![budget("TRANSFER", "Transfers", BudgetDirection::Transfer)];

    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(-25, 0), "TRANSFER", "Transfers", &budgets,),
        Decimal::new(-25, 0)
    );
    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(25, 0), "TRANSFER", "Transfers", &budgets,),
        Decimal::new(25, 0)
    );
}

#[test]
fn simple_fake_transaction_budget_code_is_inferred_from_category() {
    let budgets = vec![
        budget("OTHER", "Other", BudgetDirection::Expense),
        budget("SALARY", "Salary", BudgetDirection::Income),
    ];

    assert_eq!(
        fake_transaction_budget_code_for_save("OTHER", "Salary", &budgets, false),
        "SALARY"
    );
    assert_eq!(
        fake_transaction_budget_code_for_save("OTHER", "Salary", &budgets, true),
        "OTHER"
    );
}

#[test]
fn simple_fake_transaction_budget_code_uses_transfer_autofill_data() {
    let budgets = vec![
        budget("OTHER", "Other", BudgetDirection::Expense),
        budget("BANK-MOVE", "Internal", BudgetDirection::Transfer),
        budget("TRANSFER", "Transfer", BudgetDirection::Transfer),
    ];

    assert_eq!(
        fake_transaction_budget_code_for_save("OTHER", "Transfers", &budgets, false),
        "TRANSFER"
    );
}

#[test]
fn fake_transaction_search_matches_all_visible_terms() {
    let mut transaction = tx("2025-04-01", -42, "Coffee beans");
    transaction.counterparty = "Market Lane".to_string();
    transaction.tags = "groceries weekend".to_string();
    transaction.notes = "shared breakfast".to_string();
    let fake = FakeTransaction { id: 1, transaction };

    assert!(fake_transaction_matches_search(
        &fake,
        &fake_transaction_search_terms("market coffee groceries")
    ));
    assert!(fake_transaction_matches_search(
        &fake,
        &fake_transaction_search_terms("42 breakfast")
    ));
    assert!(!fake_transaction_matches_search(
        &fake,
        &fake_transaction_search_terms("rent")
    ));
}

#[test]
fn fake_transaction_direction_falls_back_to_code_and_category_context() {
    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(-25, 0), "INC-BONUS", "Bonus", &[]),
        Decimal::new(25, 0)
    );
    assert_eq!(
        fake_transaction_amount_for_budget(Decimal::new(25, 0), "MISC", "Other", &[]),
        Decimal::new(-25, 0)
    );
}

#[test]
fn fake_store_add_assigns_stable_ids_and_counts() {
    let store = FakeTransactionStore::new();
    let first = store.add(tx("2025-01-01", -10, "first"));
    let second = store.add(tx("2025-01-02", -20, "second"));

    assert_eq!(first, 1);
    assert_eq!(second, 2);
    assert_eq!(store.count(), 2);
    assert_eq!(store.list()[0].transaction.transaction_id, "FAKE-1");
    assert_eq!(store.list()[1].transaction.transaction_id, "FAKE-2");
}

#[test]
fn fake_store_update_remove_and_clear_work() {
    let store = FakeTransactionStore::new();
    let first = store.add(tx("2025-01-01", -10, "first"));
    let second = store.add(tx("2025-01-02", -20, "second"));

    assert!(store.update(first, tx("2025-01-03", -30, "updated")));
    assert_eq!(store.get(first).unwrap().transaction.description, "updated");
    assert!(store.remove(second));
    assert_eq!(store.count(), 1);
    assert_eq!(store.clear(), 1);
    assert_eq!(store.count(), 0);
}

#[test]
fn merged_data_appends_fakes_without_mutating_real_data() {
    let real = tx("2025-01-01", -10, "real");
    let fake = FakeTransaction {
        id: 1,
        transaction: normalize_fake_transaction(1, tx("2026-02-03", -20, "fake")),
    };
    let data = AppData {
        transactions: vec![real],
        available_months: vec![MonthKey::new(2025, 1)],
        available_years: vec![2025],
        default_month: Some(MonthKey::new(2025, 1)),
        ..AppData::default()
    };

    let merged = data_with_fake_transactions(data.clone(), vec![fake]);

    assert_eq!(data.transactions.len(), 1);
    assert_eq!(merged.transactions.len(), 2);
    assert_eq!(merged.transactions[0].description, "fake");
    assert_eq!(
        merged.available_months,
        vec![MonthKey::new(2025, 1), MonthKey::new(2026, 2)]
    );
    assert_eq!(merged.available_years, vec![2025, 2026]);
    assert!(transaction_is_fake(&merged.transactions[0]));
}

#[test]
fn real_transactions_excludes_runtime_fakes_for_export() {
    let real = tx("2025-01-01", -10, "real");
    let fake = normalize_fake_transaction(1, tx("2025-01-02", -20, "fake"));

    let real_only = real_transactions(&[real.clone(), fake]);

    assert_eq!(real_only.len(), 1);
    assert_eq!(real_only[0].description, real.description);
}
