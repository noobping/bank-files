use super::*;

mod keys;

#[derive(Clone, Default)]
pub(in crate::app) struct Preferences {
    settings: Option<gtk::gio::Settings>,
}

impl Preferences {
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

    pub(in crate::app) fn dedupe_enabled(&self) -> bool {
        self.boolean("dedupe-enabled", true)
    }

    pub(in crate::app) fn set_dedupe_enabled(&self, enabled: bool) {
        self.set_boolean("dedupe-enabled", enabled);
    }

    pub(in crate::app) fn hide_refunded_transactions(&self) -> bool {
        self.boolean("hide-refunded-transactions", true)
    }

    pub(in crate::app) fn set_hide_refunded_transactions(&self, enabled: bool) {
        self.set_boolean("hide-refunded-transactions", enabled);
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
mod tests;
