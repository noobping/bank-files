use super::*;

impl Preferences {
    pub(in crate::app) const WRITABLE_KEYS: [&'static str; 13] = [
        "active-tab",
        "autohide-status-bar",
        "show-all",
        "compare-categories-previous-period",
        "advanced-autofill",
        "advanced-features",
        "remember-mode",
        "dedupe-enabled",
        "selected-year",
        "selected-budget-month",
        "window-width",
        "window-height",
        "window-maximized",
    ];

    pub(in crate::app) fn key_for_action(action_name: &str) -> Option<&'static str> {
        match action_name.strip_prefix("app.").unwrap_or(action_name) {
            "autohide-status" => Some("autohide-status-bar"),
            "show-all" => Some("show-all"),
            "compare-categories-previous-period" => Some("compare-categories-previous-period"),
            "advanced-autofill" => Some("advanced-autofill"),
            "advanced-features" => Some("advanced-features"),
            "remember-mode" => Some("remember-mode"),
            "dedupe-enabled" => Some("dedupe-enabled"),
            _ => None,
        }
    }
}
