use super::*;

impl Preferences {
    #[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
    pub(in crate::app) const WRITABLE_KEYS: [&'static str; 17] = [
        "active-tab",
        "autohide-status-bar",
        "show-all",
        "show-predictions",
        "online-smart-insights",
        "compare-categories-previous-period",
        "advanced-autofill",
        "advanced-features",
        "remember-mode",
        "auto-clean-config",
        "dedupe-enabled",
        "hide-canceled-transactions",
        "selected-year",
        "selected-budget-month",
        "window-width",
        "window-height",
        "window-maximized",
    ];

    #[cfg(all(feature = "smart-insights", feature = "flatpak"))]
    pub(in crate::app) const WRITABLE_KEYS: [&'static str; 16] = [
        "active-tab",
        "autohide-status-bar",
        "show-all",
        "show-predictions",
        "compare-categories-previous-period",
        "advanced-autofill",
        "advanced-features",
        "remember-mode",
        "auto-clean-config",
        "dedupe-enabled",
        "hide-canceled-transactions",
        "selected-year",
        "selected-budget-month",
        "window-width",
        "window-height",
        "window-maximized",
    ];

    #[cfg(not(feature = "smart-insights"))]
    pub(in crate::app) const WRITABLE_KEYS: [&'static str; 14] = [
        "active-tab",
        "autohide-status-bar",
        "show-all",
        "compare-categories-previous-period",
        "advanced-autofill",
        "advanced-features",
        "remember-mode",
        "auto-clean-config",
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
            #[cfg(feature = "smart-insights")]
            "show-predictions" => Some("show-predictions"),
            #[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
            "online-smart-insights" => Some("online-smart-insights"),
            "compare-categories-previous-period" => Some("compare-categories-previous-period"),
            "advanced-autofill" => Some("advanced-autofill"),
            "advanced-features" => Some("advanced-features"),
            "remember-mode" => Some("remember-mode"),
            "auto-clean-config" => Some("auto-clean-config"),
            "dedupe-enabled" => Some("dedupe-enabled"),
            #[cfg(feature = "smart-insights")]
            "hide-canceled-transactions" => Some("hide-canceled-transactions"),
            _ => None,
        }
    }
}
