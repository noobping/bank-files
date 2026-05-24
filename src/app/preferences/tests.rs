use super::*;

#[test]
fn remember_mode_uses_data_and_analytics_by_default() {
    assert_eq!(
        RememberMode::from_settings(""),
        RememberMode::DataAndAnalytics
    );
    assert_eq!(RememberMode::default().as_settings(), "data-and-analytics");
}

#[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
#[test]
fn online_smart_insights_action_maps_to_preference_key() {
    assert_eq!(
        Preferences::key_for_action("app.online-smart-insights"),
        Some("online-smart-insights")
    );
}

#[cfg(any(not(feature = "smart-insights"), feature = "flatpak"))]
#[test]
fn online_smart_insights_action_is_not_available_without_hosted_smart_insights() {
    assert_eq!(
        Preferences::key_for_action("app.online-smart-insights"),
        None
    );
}

#[test]
fn spending_comparison_action_maps_to_preference_key() {
    assert_eq!(
        Preferences::key_for_action("app.compare-categories-previous-period"),
        Some("compare-categories-previous-period")
    );
}

#[cfg(not(feature = "smart-insights"))]
#[test]
fn smart_insights_actions_are_not_available_without_feature() {
    assert_eq!(Preferences::key_for_action("app.show-predictions"), None);
    assert_eq!(
        Preferences::key_for_action("app.hide-canceled-transactions"),
        None
    );
}
