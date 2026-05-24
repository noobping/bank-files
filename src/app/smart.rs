pub(in crate::app) fn smart_insights_available() -> bool {
    cfg!(feature = "smart-insights")
}

pub(in crate::app) fn smart_pattern_detection_enabled(
    advanced_features: bool,
    show_predictions: bool,
) -> bool {
    smart_insights_available() && advanced_features && show_predictions
}

#[cfg(feature = "smart-insights")]
pub(in crate::app) fn smart_dependent_action_enabled(
    advanced_features: bool,
    show_predictions: bool,
    writable: bool,
) -> bool {
    smart_pattern_detection_enabled(advanced_features, show_predictions) && writable
}

pub(in crate::app) fn effective_hide_canceled_transactions(
    advanced_features: bool,
    show_predictions: bool,
    hide_canceled_transactions: bool,
) -> bool {
    smart_pattern_detection_enabled(advanced_features, show_predictions)
        && hide_canceled_transactions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_detection_requires_feature_advanced_mode_and_preference() {
        assert_eq!(
            smart_pattern_detection_enabled(true, true),
            cfg!(feature = "smart-insights")
        );
        assert!(!smart_pattern_detection_enabled(false, true));
        assert!(!smart_pattern_detection_enabled(true, false));
    }

    #[cfg(feature = "smart-insights")]
    #[test]
    fn smart_dependent_actions_require_feature_advanced_preference_and_writable_settings() {
        assert_eq!(
            smart_dependent_action_enabled(true, true, true),
            cfg!(feature = "smart-insights")
        );
        assert!(!smart_dependent_action_enabled(false, true, true));
        assert!(!smart_dependent_action_enabled(true, false, true));
        assert!(!smart_dependent_action_enabled(true, true, false));
    }

    #[test]
    fn refunded_transaction_hiding_requires_feature_advanced_preference_and_setting() {
        assert_eq!(
            effective_hide_canceled_transactions(true, true, true),
            cfg!(feature = "smart-insights")
        );
        assert!(!effective_hide_canceled_transactions(false, true, true));
        assert!(!effective_hide_canceled_transactions(true, false, true));
        assert!(!effective_hide_canceled_transactions(true, true, false));
    }
}
