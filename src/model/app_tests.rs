use super::*;

#[test]
fn all_scope_satisfies_narrower_scopes() {
    assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Year(Some(2025))));
    assert!(TransactionLoadScope::All
        .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 5),))));
    assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Year(None)));
    assert!(!TransactionLoadScope::All.satisfies(TransactionLoadScope::Unloaded));
}

#[test]
fn remember_mode_retention_levels_are_ordered() {
    assert!(RememberMode::Forget.retains_less_than(RememberMode::DataOnly));
    assert!(RememberMode::DataOnly.retains_less_than(RememberMode::DataAndAnalytics));
    assert!(RememberMode::Forget.retains_less_than(RememberMode::DataAndAnalytics));
    assert!(!RememberMode::DataAndAnalytics.retains_less_than(RememberMode::DataOnly));
}

#[test]
fn year_scope_satisfies_months_inside_that_year() {
    assert!(TransactionLoadScope::Year(Some(2025))
        .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))));
    assert!(!TransactionLoadScope::Year(Some(2025))
        .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2024, 12)))));
    assert!(!TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))
        .satisfies(TransactionLoadScope::Year(Some(2025))));
}

#[test]
fn previous_period_scopes_cover_current_and_previous_periods() {
    assert!(
        TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2025, 5)))
            .satisfies(TransactionLoadScope::Month(Some(MonthKey::new(2025, 4))))
    );
    assert!(TransactionLoadScope::YearWithPrevious(Some(2025))
        .satisfies(TransactionLoadScope::Year(Some(2024))));
    assert!(
        TransactionLoadScope::YearWithPrevious(Some(2025)).satisfies(
            TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2025, 1)))
        )
    );
    assert!(
        !TransactionLoadScope::YearWithPrevious(Some(2025)).satisfies(
            TransactionLoadScope::MonthWithPrevious(Some(MonthKey::new(2024, 1)))
        )
    );
}

#[test]
fn unresolved_scopes_only_satisfy_exact_or_all_scopes() {
    assert!(TransactionLoadScope::Year(None).satisfies(TransactionLoadScope::Year(None)));
    assert!(TransactionLoadScope::All.satisfies(TransactionLoadScope::Month(None)));
    assert!(!TransactionLoadScope::Year(Some(2025)).satisfies(TransactionLoadScope::Year(None)));
    assert!(!TransactionLoadScope::Month(Some(MonthKey::new(2025, 5)))
        .satisfies(TransactionLoadScope::Month(None)));
}
