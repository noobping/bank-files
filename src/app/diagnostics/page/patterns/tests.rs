use super::*;

#[test]
fn transaction_patterns_need_all_reload_only_for_partial_scopes() {
    let all = TransactionLoadScope::All;
    let year = TransactionLoadScope::Year(Some(2025));
    let month = TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)));

    assert!(!transaction_patterns_need_all_reload(all));
    assert!(transaction_patterns_need_all_reload(year));
    assert!(transaction_patterns_need_all_reload(month));
}

#[test]
fn transaction_patterns_section_requires_smart_insights() {
    assert!(transaction_patterns_section_visible(None, true));
    assert!(!transaction_patterns_section_visible(None, false));

    let pattern_search = SearchFilter::from_text("patterns").unwrap();
    assert!(transaction_patterns_section_visible(
        Some(&pattern_search),
        true
    ));
    assert!(!transaction_patterns_section_visible(
        Some(&pattern_search),
        false
    ));
}

#[test]
fn transaction_patterns_section_still_respects_search_terms() {
    let warnings_search = SearchFilter::from_text("warnings").unwrap();

    assert!(!transaction_patterns_section_visible(
        Some(&warnings_search),
        true
    ));
}

#[test]
fn patterns_preset_matches_pattern_section() {
    let pattern_search = SearchFilter::from_text("patterns").unwrap();
    let unrelated_search = SearchFilter::from_text("warnings").unwrap();

    assert!(transaction_patterns_section_matches(Some(&pattern_search)));
    assert!(!transaction_patterns_section_matches(Some(
        &unrelated_search
    )));
}
