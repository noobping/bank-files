pub(in crate::app) fn smart_pattern_detection_enabled(show_predictions: bool) -> bool {
    show_predictions
}

pub(in crate::app) fn effective_hide_canceled_transactions(
    show_predictions: bool,
    hide_canceled_transactions: bool,
) -> bool {
    smart_pattern_detection_enabled(show_predictions) && hide_canceled_transactions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_detection_follows_smart_insights() {
        assert!(smart_pattern_detection_enabled(true));
        assert!(!smart_pattern_detection_enabled(false));
    }

    #[test]
    fn refunded_transaction_hiding_requires_smart_insights() {
        assert!(effective_hide_canceled_transactions(true, true));
        assert!(!effective_hide_canceled_transactions(false, true));
        assert!(!effective_hide_canceled_transactions(true, false));
    }
}
