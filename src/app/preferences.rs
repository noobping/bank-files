use super::*;

#[derive(Clone, Default)]
pub(in crate::app) struct Preferences {
    settings: Option<gtk::gio::Settings>,
}

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

    pub(in crate::app) fn new() -> Self {
        let settings = gtk::gio::SettingsSchemaSource::default()
            .and_then(|source| source.lookup(APP_ID, true))
            .map(|schema| {
                gtk::gio::Settings::new_full(&schema, None::<&gtk::gio::SettingsBackend>, None)
            });
        Self { settings }
    }

    pub(in crate::app) fn active_tab(&self) -> String {
        let tab = self.string("active-tab", "overview");
        match tab.as_str() {
            "overview" | "categories" | "transactions" | "debug" => tab,
            _ => "overview".to_string(),
        }
    }

    pub(in crate::app) fn set_active_tab(&self, tab: &str) {
        if matches!(tab, "overview" | "categories" | "transactions" | "debug") {
            self.set_string("active-tab", tab);
        }
    }

    pub(in crate::app) fn autohide_status_bar(&self) -> bool {
        self.boolean("autohide-status-bar", false)
    }

    pub(in crate::app) fn set_autohide_status_bar(&self, enabled: bool) {
        self.set_boolean("autohide-status-bar", enabled);
    }

    pub(in crate::app) fn show_all(&self) -> bool {
        self.boolean("show-all", false)
    }

    pub(in crate::app) fn set_show_all(&self, enabled: bool) {
        self.set_boolean("show-all", enabled);
    }

    pub(in crate::app) fn show_predictions(&self) -> bool {
        cfg!(feature = "smart-insights") && self.boolean("show-predictions", false)
    }

    #[cfg(feature = "smart-insights")]
    pub(in crate::app) fn set_show_predictions(&self, enabled: bool) {
        self.set_boolean("show-predictions", enabled);
    }

    #[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
    pub(in crate::app) fn online_smart_insights(&self) -> bool {
        self.boolean("online-smart-insights", false)
    }

    #[cfg(all(not(feature = "smart-insights"), not(feature = "flatpak")))]
    pub(in crate::app) fn online_smart_insights(&self) -> bool {
        false
    }

    #[cfg(all(feature = "smart-insights", not(feature = "flatpak")))]
    pub(in crate::app) fn set_online_smart_insights(&self, enabled: bool) {
        self.set_boolean("online-smart-insights", enabled);
    }

    pub(in crate::app) fn compare_categories_previous_period(&self) -> bool {
        self.boolean("compare-categories-previous-period", false)
    }

    pub(in crate::app) fn set_compare_categories_previous_period(&self, enabled: bool) {
        self.set_boolean("compare-categories-previous-period", enabled);
    }

    pub(in crate::app) fn advanced_autofill(&self) -> bool {
        self.boolean("advanced-autofill", true)
    }

    pub(in crate::app) fn set_advanced_autofill(&self, enabled: bool) {
        self.set_boolean("advanced-autofill", enabled);
    }

    pub(in crate::app) fn advanced_features(&self) -> bool {
        self.boolean("advanced-features", false)
    }

    pub(in crate::app) fn set_advanced_features(&self, enabled: bool) {
        self.set_boolean("advanced-features", enabled);
    }

    pub(in crate::app) fn remember_mode(&self) -> RememberMode {
        RememberMode::from_settings(
            &self.string("remember-mode", RememberMode::default().as_settings()),
        )
    }

    pub(in crate::app) fn set_remember_mode(&self, mode: RememberMode) {
        self.set_string("remember-mode", mode.as_settings());
    }

    pub(in crate::app) fn auto_clean_config(&self) -> bool {
        self.boolean("auto-clean-config", false)
    }

    pub(in crate::app) fn set_auto_clean_config(&self, enabled: bool) {
        self.set_boolean("auto-clean-config", enabled);
    }

    pub(in crate::app) fn dedupe_enabled(&self) -> bool {
        self.boolean("dedupe-enabled", true)
    }

    pub(in crate::app) fn set_dedupe_enabled(&self, enabled: bool) {
        self.set_boolean("dedupe-enabled", enabled);
    }

    pub(in crate::app) fn hide_canceled_transactions(&self) -> bool {
        cfg!(feature = "smart-insights") && self.boolean("hide-canceled-transactions", false)
    }

    #[cfg(feature = "smart-insights")]
    pub(in crate::app) fn set_hide_canceled_transactions(&self, enabled: bool) {
        self.set_boolean("hide-canceled-transactions", enabled);
    }

    pub(in crate::app) fn selected_year(&self) -> Option<i32> {
        let year = self.int("selected-year", 0);
        (year > 0).then_some(year)
    }

    pub(in crate::app) fn set_selected_year(&self, year: i32) {
        self.set_int("selected-year", year.max(0));
    }

    pub(in crate::app) fn selected_budget_month(&self) -> Option<MonthKey> {
        parse_month_key(&self.string("selected-budget-month", ""))
    }

    pub(in crate::app) fn set_selected_budget_month(&self, month: MonthKey) {
        self.set_string("selected-budget-month", &month.to_string());
    }

    pub(in crate::app) fn window_width(&self) -> i32 {
        self.int("window-width", 1250).max(640)
    }

    pub(in crate::app) fn window_height(&self) -> i32 {
        self.int("window-height", 820).max(480)
    }

    pub(in crate::app) fn window_maximized(&self) -> bool {
        self.boolean("window-maximized", false)
    }

    pub(in crate::app) fn set_window_state(&self, width: i32, height: i32, maximized: bool) {
        self.set_int("window-width", width.max(640));
        self.set_int("window-height", height.max(480));
        self.set_boolean("window-maximized", maximized);
    }

    pub(in crate::app) fn is_writable(&self, key: &str) -> bool {
        self.settings
            .as_ref()
            .map(|settings| settings.is_writable(key))
            .unwrap_or(true)
    }

    pub(in crate::app) fn any_writable(&self) -> bool {
        Self::WRITABLE_KEYS.iter().any(|key| self.is_writable(key))
    }

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

    pub(in crate::app) fn action_is_writable(&self, action_name: &str) -> bool {
        Self::key_for_action(action_name)
            .map(|key| self.is_writable(key))
            .unwrap_or(true)
    }

    fn boolean(&self, key: &str, fallback: bool) -> bool {
        self.settings
            .as_ref()
            .map(|settings| settings.boolean(key))
            .unwrap_or(fallback)
    }

    fn int(&self, key: &str, fallback: i32) -> i32 {
        self.settings
            .as_ref()
            .map(|settings| settings.int(key))
            .unwrap_or(fallback)
    }

    fn string(&self, key: &str, fallback: &str) -> String {
        self.settings
            .as_ref()
            .map(|settings| settings.string(key).to_string())
            .unwrap_or_else(|| fallback.to_string())
    }

    fn set_boolean(&self, key: &str, value: bool) {
        if let Some(settings) = &self.settings {
            if settings.is_writable(key) {
                let _ = settings.set_boolean(key, value);
            }
        }
    }

    fn set_int(&self, key: &str, value: i32) {
        if let Some(settings) = &self.settings {
            if settings.is_writable(key) {
                let _ = settings.set_int(key, value);
            }
        }
    }

    fn set_string(&self, key: &str, value: &str) {
        if let Some(settings) = &self.settings {
            if settings.is_writable(key) {
                let _ = settings.set_string(key, value);
            }
        }
    }
}

fn parse_month_key(input: &str) -> Option<MonthKey> {
    let (year, month) = input.trim().split_once('-')?;
    let year = year.parse::<i32>().ok()?;
    let month = month.parse::<u32>().ok()?;
    (1..=12)
        .contains(&month)
        .then_some(MonthKey::new(year, month))
}

#[cfg(test)]
mod tests {
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
}
