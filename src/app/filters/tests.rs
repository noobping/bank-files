use super::page::AppPage;
use super::presets::{search_preset_specs, SearchPreset};
use super::search::SearchFilter;
use super::transaction_filter::{TransactionAmountFilter, TransactionFilter};
use crate::model::{
    BudgetAmount, BudgetCode, BudgetDirection, BudgetIncomeBasis, BudgetSpecialKind, MonthKey,
    Transaction,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[test]
fn transaction_filter_with_year_preserves_month_and_details() {
    let filter = TransactionFilter::Scoped {
        budget_code: Some("FOOD".to_string()),
        year: None,
        month: Some(MonthKey::new(2025, 5)),
        amount: Some(TransactionAmountFilter::Expense),
    };

    let shifted = filter.with_year(2024).unwrap();

    assert!(matches!(
        shifted,
        TransactionFilter::Scoped {
            budget_code: Some(ref code),
            year: None,
            month: Some(MonthKey { year: 2024, month: 5 }),
            amount: Some(TransactionAmountFilter::Expense),
        } if code == "FOOD"
    ));
}

#[test]
fn transaction_filter_with_year_preserves_category_filter() {
    let filter = TransactionFilter::category_for_year("Groceries", 2025);

    let shifted = filter.with_year(2024).unwrap();

    assert!(matches!(
        shifted,
        TransactionFilter::CategoryForYear { ref category, year: 2024 }
            if category == "Groceries"
    ));
}

#[test]
fn transaction_amount_filter_aliases_are_parsed() {
    assert!(matches!(
        TransactionFilter::from_query("amount:positive"),
        Some(TransactionFilter::Scoped {
            amount: Some(TransactionAmountFilter::Income),
            ..
        })
    ));
    assert!(matches!(
        TransactionFilter::from_query("amount:costs"),
        Some(TransactionFilter::Scoped {
            amount: Some(TransactionAmountFilter::Expense),
            ..
        })
    ));
    assert!(matches!(
        TransactionFilter::from_query("amount:transfer"),
        Some(TransactionFilter::Scoped {
            amount: Some(TransactionAmountFilter::Transfer),
            ..
        })
    ));
    assert!(matches!(
        TransactionFilter::from_query("amount:refunded"),
        Some(TransactionFilter::Scoped {
            amount: Some(TransactionAmountFilter::Refund),
            ..
        })
    ));
}

#[test]
fn transaction_special_amount_filters_round_trip() {
    let Some(filter) = TransactionFilter::from_query("amount:transfer") else {
        panic!("transfer amount filter should parse");
    };

    assert_eq!(filter.query(), "amount:transfer");

    let Some(filter) = TransactionFilter::from_query("amount:refund") else {
        panic!("refund amount filter should parse");
    };

    assert_eq!(filter.query(), "amount:refund");
    assert!(filter.shows_refunds());
}

#[test]
fn budget_transaction_filter_matches_child_budget_codes() {
    let budgets = vec![
        budget("HOME", "", "Housing"),
        budget("RENT", "HOME", "Rent"),
    ];
    let filter = TransactionFilter::budget_for_year("HOME", 2025);

    assert!(filter.matches(&tx("2025-01-01", "HOME"), &budgets));
    assert!(filter.matches(&tx("2025-01-02", "RENT"), &budgets));
    assert!(!filter.matches(&tx("2025-01-03", "FOOD"), &budgets));
}

#[test]
fn structured_search_filters_match_summary_cards_after_data_filtering() {
    let amount_filter = SearchFilter::from_text("amount:income").unwrap();
    let text_filter = SearchFilter::from_text("groceries").unwrap();

    assert!(amount_filter.matches_summary("rent budget"));
    assert!(!text_filter.matches_summary("rent budget"));
}

#[test]
fn search_preset_specs_have_unique_ids_and_known_queries() {
    let mut ids = std::collections::BTreeSet::new();

    for spec in search_preset_specs() {
        assert!(ids.insert(spec.id));
        assert!(SearchPreset::from_id(spec.id).is_some());
    }
}

#[test]
fn search_presets_choose_their_most_useful_page() {
    assert_eq!(SearchPreset::Clear.target_page(), None);
    assert_eq!(
        SearchPreset::Income.target_page(),
        Some(AppPage::Transactions)
    );
    assert_eq!(
        SearchPreset::UnconfiguredBudgets.target_page(),
        Some(AppPage::Transactions)
    );
    assert_eq!(
        SearchPreset::Warnings.target_page(),
        Some(AppPage::Diagnostics)
    );
}

#[test]
fn app_pages_round_trip_stack_names() {
    for page in [
        AppPage::Overview,
        AppPage::Budget,
        AppPage::Transactions,
        AppPage::Diagnostics,
    ] {
        assert_eq!(AppPage::from_stack_name(Some(page.stack_name())), page);
    }
}

fn budget(code: &str, parent_code: &str, category: &str) -> BudgetCode {
    BudgetCode {
        code: code.to_string(),
        parent_code: parent_code.to_string(),
        special: BudgetSpecialKind::None,
        category: category.to_string(),
        monthly_budget: Some(BudgetAmount::Fixed(Decimal::new(100, 0))),
        yearly_budget: None,
        direction: BudgetDirection::Expense,
        income_basis: BudgetIncomeBasis::RealIncome,
        notes: String::new(),
    }
}

fn tx(date: &str, budget_code: &str) -> Transaction {
    Transaction {
        date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        amount: Decimal::new(-10, 0),
        description: String::new(),
        tags: String::new(),
        counterparty: String::new(),
        account: String::new(),
        transaction_id: String::new(),
        currency: String::new(),
        source_file: String::new(),
        source_row: 1,
        category: String::new(),
        budget_code: budget_code.to_string(),
        notes: String::new(),
        strict_key: String::new(),
        loose_key: String::new(),
        rule_match: None,
    }
}
