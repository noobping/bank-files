use super::page::AppPage;
use super::presets::{search_preset_specs, SearchPreset};
use super::search::SearchFilter;
use super::transaction_filter::{TransactionAmountFilter, TransactionFilter};
use crate::model::MonthKey;

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
}

#[test]
fn transaction_transfer_filter_round_trips() {
    let Some(filter) = TransactionFilter::from_query("amount:transfer") else {
        panic!("transfer amount filter should parse");
    };

    assert_eq!(filter.query(), "amount:transfer");
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
