use super::*;

#[test]
fn remember_mode_uses_data_and_analytics_by_default() {
    assert_eq!(
        RememberMode::from_settings(""),
        RememberMode::DataAndAnalytics
    );
    assert_eq!(RememberMode::default().as_settings(), "data-and-analytics");
}

#[test]
fn preference_actions_map_to_preference_keys() {
    assert_eq!(
        Preferences::key_for_action("app.compare-categories-previous-period"),
        Some("compare-categories-previous-period")
    );
    assert_eq!(
        Preferences::key_for_action("app.hide-refunded-transactions"),
        Some("hide-refunded-transactions")
    );
}
