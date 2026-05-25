use super::group::preference_row_visible;
use super::snapshot::preferences_page_snapshot;
use super::*;

#[test]
fn preference_row_visibility_follows_managed_state_and_mode() {
    assert!(preference_row_visible(true, false));
    assert!(preference_row_visible(true, true));
    assert!(!preference_row_visible(false, false));
    assert!(preference_row_visible(false, true));
}

#[test]
fn spending_comparison_is_an_interface_preference() {
    let preferences = Preferences::default();
    let snapshot = preferences_page_snapshot(false, &preferences);

    assert!(snapshot.rows().iter().any(|row| {
        row[0] == tr("Interface") && row[1] == tr("Compare Spending with Previous Period")
    }));
}

#[test]
fn whole_form_autofill_is_a_forms_and_data_preference() {
    let preferences = Preferences::default();
    let snapshot = preferences_page_snapshot(false, &preferences);

    assert!(snapshot
        .rows()
        .iter()
        .any(|row| { row[0] == tr("Forms and Data") && row[1] == tr("Whole Form Autofill") }));
}

#[test]
fn hide_refunded_transactions_is_a_forms_and_data_preference() {
    let preferences = Preferences::default();
    let snapshot = preferences_page_snapshot(false, &preferences);

    assert!(snapshot.rows().iter().any(|row| {
        row[0] == tr("Forms and Data") && row[1] == tr("Hide Refunded Transactions")
    }));
}
