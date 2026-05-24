use super::group::{preference_row_enabled, preference_row_visible};
use super::snapshot::preferences_page_snapshot;
use super::*;

#[test]
fn preference_row_visibility_follows_managed_state_and_mode() {
    assert!(preference_row_visible(true, false, false, false));
    assert!(preference_row_visible(true, true, false, false));
    assert!(!preference_row_visible(false, false, false, false));
    assert!(preference_row_visible(false, true, false, false));
}

#[test]
fn smart_dependent_preferences_follow_smart_state_and_advanced_override() {
    assert!(preference_row_visible(true, false, true, true));
    assert!(!preference_row_visible(true, false, false, true));
    assert!(preference_row_visible(true, true, false, true));
}

#[test]
fn experimental_preferences_are_advanced_only() {
    let preferences = Preferences::default();
    let simple_snapshot = preferences_page_snapshot(false, true, &preferences);
    let advanced_snapshot = preferences_page_snapshot(true, false, &preferences);

    assert!(!simple_snapshot
        .rows()
        .iter()
        .any(|row| row[0] == tr("Experimental")));
    #[cfg(feature = "smart-insights")]
    assert!(advanced_snapshot
        .rows()
        .iter()
        .any(|row| row[0] == tr("Experimental")));
    #[cfg(not(feature = "smart-insights"))]
    assert!(!advanced_snapshot
        .rows()
        .iter()
        .any(|row| row[0] == tr("Experimental")));
}

#[test]
fn spending_comparison_is_an_interface_preference() {
    let preferences = Preferences::default();
    let snapshot = preferences_page_snapshot(false, false, &preferences);

    assert!(snapshot.rows().iter().any(|row| {
        row[0] == tr("Interface") && row[1] == tr("Compare Spending with Previous Period")
    }));
    assert!(!snapshot.rows().iter().any(|row| {
        row[0] == tr("Experimental") && row[1] == tr("Compare Spending with Previous Period")
    }));
}

#[test]
fn whole_form_autofill_is_a_forms_and_data_preference() {
    let preferences = Preferences::default();
    let snapshot = preferences_page_snapshot(false, false, &preferences);

    assert!(snapshot
        .rows()
        .iter()
        .any(|row| { row[0] == tr("Forms and Data") && row[1] == tr("Whole Form Autofill") }));
    assert!(!snapshot
        .rows()
        .iter()
        .any(|row| { row[0] == tr("Experimental") && row[1] == tr("Whole Form Autofill") }));
}

#[test]
fn smart_dependent_preferences_disable_without_smart_insights() {
    assert!(preference_row_enabled(true, true, true));
    assert!(!preference_row_enabled(true, false, true));
    assert!(!preference_row_enabled(false, true, true));
}
