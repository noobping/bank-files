use super::*;

pub(in crate::app) fn show_preferences_dialog(
    parent: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let shell = build_settings_dialog_shell("Preferences", "Search preferences");
    let root = shell.root;
    let search_button = shell.search_button;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let page = adw::PreferencesPage::builder()
        .title(tr("General"))
        .icon_name("preferences-system-symbolic")
        .build();
    let mut search_groups = Vec::new();
    let smart_insights_enabled = ui.show_predictions.get();

    if let Some((group, search_group)) = preference_group(
        "Interface",
        "Control how much information is visible while you browse.",
        &[
            PreferenceSpec::new(
                "Autohide Status Bar",
                "Hide status messages automatically after a short delay.",
                "app.autohide-status",
                ui.status_autohide.get(),
            ),
            PreferenceSpec::new(
                "Always Show Full Lists",
                "Show every item immediately and hide More buttons.",
                "app.show-all",
                ui.show_all.get(),
            ),
        ],
        ui.advanced_features.get(),
        smart_insights_enabled,
        &ui.preferences,
    ) {
        page.add(&group);
        search_groups.push(search_group);
    }

    #[cfg(feature = "smart-insights")]
    let mut insight_preferences = Vec::new();
    #[cfg(not(feature = "smart-insights"))]
    let insight_preferences = Vec::new();
    #[cfg(feature = "smart-insights")]
    {
        insight_preferences.push(PreferenceSpec::new(
            "Smart Insights",
            "Show forecast cards and detect transaction patterns, including possible transfers, from imported transactions.",
            "app.show-predictions",
            ui.show_predictions.get(),
        ));
        #[cfg(not(feature = "flatpak"))]
        insight_preferences.push(
            PreferenceSpec::new(
                "Online Smart Insights",
                "Allow privacy-filtered company category lookups. Amounts, dates, accounts, descriptions, notes, and rows are never sent.",
                "app.online-smart-insights",
                ui.online_smart_insights.get(),
            )
            .requires_smart_insights(),
        );
        insight_preferences.push(PreferenceSpec::new(
            "Compare Spending with Previous Period",
            "Compare spending cards with the previous month or year.",
            "app.compare-categories-previous-period",
            ui.compare_categories_previous_period.get(),
        ));
    }

    if let Some((group, search_group)) = preference_group(
        "Insights",
        "Control smart forecasts, detected transaction patterns, smart transfer detection, and previous-period spending comparisons.",
        &insight_preferences,
        ui.advanced_features.get(),
        smart_insights_enabled,
        &ui.preferences,
    ) {
        page.add(&group);
        search_groups.push(search_group);
    }

    if let Some((group, search_group)) =
        remember_preference_group(ui.advanced_features.get(), state, ui)
    {
        page.add(&group);
        search_groups.push(search_group);
    }

    #[cfg(feature = "smart-insights")]
    let mut forms_preferences = vec![
        PreferenceSpec::new(
            "Advanced Features",
            "Allow rule editing and budget direction controls.",
            "app.advanced-features",
            ui.advanced_features.get(),
        ),
        PreferenceSpec::new(
            "Smart Autofill",
            "Let forms fill related fields from context, such as matching categories and directions.",
            "app.advanced-autofill",
            ui.advanced_autofill.get(),
        ),
        PreferenceSpec::new(
            "Auto Clean Config",
            "Remove orphaned rules automatically during reload and import.",
            "app.auto-clean-config",
            ui.auto_clean_config.get(),
        ),
    ];
    #[cfg(not(feature = "smart-insights"))]
    let forms_preferences = vec![
        PreferenceSpec::new(
            "Advanced Features",
            "Allow rule editing and budget direction controls.",
            "app.advanced-features",
            ui.advanced_features.get(),
        ),
        PreferenceSpec::new(
            "Smart Autofill",
            "Let forms fill related fields from context, such as matching categories and directions.",
            "app.advanced-autofill",
            ui.advanced_autofill.get(),
        ),
        PreferenceSpec::new(
            "Auto Clean Config",
            "Remove orphaned rules automatically during reload and import.",
            "app.auto-clean-config",
            ui.auto_clean_config.get(),
        ),
    ];
    #[cfg(feature = "smart-insights")]
    forms_preferences.push(
        PreferenceSpec::new(
            "Hide Refunded Transactions",
            "Requires Smart Insights. Exclude detected refunds and offsetting groups from normal views.",
            "app.hide-canceled-transactions",
            ui.hide_canceled_transactions.get(),
        )
        .requires_smart_insights(),
    );

    if let Some((group, search_group)) = preference_group(
        "Forms and Data",
        "Control simple mode, smart form filling, cleanup, and refunded transaction visibility.",
        &forms_preferences,
        ui.advanced_features.get(),
        smart_insights_enabled,
        &ui.preferences,
    ) {
        page.add(&group);
        search_groups.push(search_group);
    }

    root.append(&ui::scroll(&page));

    let status_bar = build_status_bar();
    connect_embedded_status_bar(parent, &status_bar, Rc::clone(&ui.status_autohide));
    connect_static_page_actions(
        &status_bar.page_actions_button,
        "preferences",
        &status_bar.label,
        ui,
        preferences_page_snapshot(
            ui.advanced_features.get(),
            smart_insights_enabled,
            &ui.preferences,
        ),
    );
    status_bar
        .label
        .set_text(&tr("Preference changes are applied immediately."));
    root.append(&status_bar.container);

    let dialog = adw::Dialog::builder()
        .title(tr("Preferences"))
        .content_width(680)
        .content_height(620)
        .child(&root)
        .build();

    ui::connect_search_button(&search_button, &search_bar, &search_entry);
    ui::connect_search_shortcut(&dialog, &search_bar, &search_entry);
    search_bar.set_key_capture_widget(Some(&dialog));
    connect_preference_search(&search_entry, search_groups);

    dialog.present(Some(parent));
}

fn preferences_page_snapshot(
    advanced_features: bool,
    smart_insights_enabled: bool,
    preferences: &Preferences,
) -> StaticPageSnapshot {
    let mut rows = Vec::new();
    add_preference_snapshot_rows(
        &mut rows,
        "Interface",
        &[
            (
                "Autohide Status Bar",
                "Hide status messages automatically after a short delay.",
                "app.autohide-status",
                false,
            ),
            (
                "Always Show Full Lists",
                "Show every item immediately and hide More buttons.",
                "app.show-all",
                false,
            ),
        ],
        advanced_features,
        smart_insights_enabled,
        preferences,
    );
    #[cfg(feature = "smart-insights")]
    let mut insight_rows = Vec::new();
    #[cfg(not(feature = "smart-insights"))]
    let insight_rows = Vec::new();
    #[cfg(feature = "smart-insights")]
    {
        insight_rows.push((
            "Smart Insights",
            "Show forecast cards and detect transaction patterns, including possible transfers, from imported transactions.",
            "app.show-predictions",
            false,
        ));
        #[cfg(not(feature = "flatpak"))]
        insight_rows.push((
            "Online Smart Insights",
            "Allow privacy-filtered company category lookups. Amounts, dates, accounts, descriptions, notes, and rows are never sent.",
            "app.online-smart-insights",
            true,
        ));
        insight_rows.push((
            "Compare Spending with Previous Period",
            "Compare spending cards with the previous month or year.",
            "app.compare-categories-previous-period",
            false,
        ));
    }
    add_preference_snapshot_rows(
        &mut rows,
        "Insights",
        &insight_rows,
        advanced_features,
        smart_insights_enabled,
        preferences,
    );
    add_preference_snapshot_rows(
        &mut rows,
        "Remember",
        &[(
            "Remember",
            "Choose whether opened CSV data is forgotten after this session, remembered as data, or remembered with reusable analysis cache.",
            "app.remember-mode",
            false,
        )],
        advanced_features,
        smart_insights_enabled,
        preferences,
    );
    #[cfg(feature = "smart-insights")]
    let mut forms_rows = vec![
        (
            "Advanced Features",
            "Allow rule editing and budget direction controls.",
            "app.advanced-features",
            false,
        ),
        (
            "Smart Autofill",
            "Let forms fill related fields from context, such as matching categories and directions.",
            "app.advanced-autofill",
            false,
        ),
        (
            "Auto Clean Config",
            "Remove orphaned rules automatically during reload and import.",
            "app.auto-clean-config",
            false,
        ),
    ];
    #[cfg(not(feature = "smart-insights"))]
    let forms_rows = vec![
        (
            "Advanced Features",
            "Allow rule editing and budget direction controls.",
            "app.advanced-features",
            false,
        ),
        (
            "Smart Autofill",
            "Let forms fill related fields from context, such as matching categories and directions.",
            "app.advanced-autofill",
            false,
        ),
        (
            "Auto Clean Config",
            "Remove orphaned rules automatically during reload and import.",
            "app.auto-clean-config",
            false,
        ),
    ];
    #[cfg(feature = "smart-insights")]
    forms_rows.push((
        "Hide Refunded Transactions",
        "Requires Smart Insights. Exclude detected refunds and offsetting groups from normal views.",
        "app.hide-canceled-transactions",
        true,
    ));
    add_preference_snapshot_rows(
        &mut rows,
        "Forms and Data",
        &forms_rows,
        advanced_features,
        smart_insights_enabled,
        preferences,
    );

    StaticPageSnapshot::new(
        "preferences",
        "Preferences",
        "Preference changes are applied immediately.",
        &["Group", "Preference", "Description"],
        rows,
    )
}

fn add_preference_snapshot_rows(
    rows: &mut Vec<Vec<String>>,
    group_title: &str,
    specs: &[(&str, &str, &str, bool)],
    advanced_features: bool,
    smart_insights_enabled: bool,
    preferences: &Preferences,
) {
    for (title, subtitle, action_name, requires_smart_insights) in specs {
        let writable = Preferences::key_for_action(action_name)
            .map(|key| preferences.is_writable(key))
            .unwrap_or(true);
        if preference_row_visible(
            writable,
            advanced_features,
            smart_insights_enabled,
            *requires_smart_insights,
        ) {
            rows.push(vec![tr(group_title), tr(title), tr(subtitle)]);
        }
    }
}

struct PreferenceSpec<'a> {
    title: &'a str,
    subtitle: &'a str,
    action_name: &'a str,
    active: bool,
    requires_smart_insights: bool,
}

impl<'a> PreferenceSpec<'a> {
    fn new(title: &'a str, subtitle: &'a str, action_name: &'a str, active: bool) -> Self {
        Self {
            title,
            subtitle,
            action_name,
            active,
            requires_smart_insights: false,
        }
    }

    #[cfg(feature = "smart-insights")]
    fn requires_smart_insights(mut self) -> Self {
        self.requires_smart_insights = true;
        self
    }

    fn visible(
        &self,
        writable: bool,
        advanced_features: bool,
        smart_insights_enabled: bool,
    ) -> bool {
        preference_row_visible(
            writable,
            advanced_features,
            smart_insights_enabled,
            self.requires_smart_insights,
        )
    }

    fn enabled(&self, writable: bool, smart_insights_enabled: bool) -> bool {
        preference_row_enabled(
            writable,
            smart_insights_enabled,
            self.requires_smart_insights,
        )
    }
}

fn remember_preference_group(
    advanced_features: bool,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let title = "Remember";
    let description = "Choose whether opened CSV data is forgotten after this session, remembered as data, or remembered with reusable analysis cache.";
    let writable = Preferences::key_for_action("app.remember-mode")
        .map(|key| ui.preferences.is_writable(key))
        .unwrap_or(true);
    if !preference_row_visible(
        writable,
        advanced_features,
        ui.show_predictions.get(),
        false,
    ) {
        return None;
    }

    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let row = remember_preference_row(state, ui, writable);
    search_group.add_row(&row, title, description);
    group.add(&row);
    Some((group, search_group))
}

fn remember_preference_row(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    writable: bool,
) -> adw::ComboRow {
    let labels = RememberMode::SETTINGS_VALUES
        .iter()
        .map(|mode| tr(mode.label()))
        .collect::<Vec<_>>();
    let label_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let model = gtk::StringList::new(&label_refs);
    let selected = remember_mode_index(ui.remember_mode.get());
    let row = adw::ComboRow::builder()
        .title(tr("Remember"))
        .subtitle(tr("Forget opens CSVs live for this session. Data only remembers copied CSVs. Data and analytics also keeps a reusable processed cache."))
        .model(&model)
        .selected(selected)
        .build();

    if writable {
        let state_for_row = Rc::clone(state);
        let ui_for_row = Rc::clone(ui);
        row.connect_selected_notify(move |row| {
            let Some(mode) = RememberMode::SETTINGS_VALUES
                .get(row.selected() as usize)
                .copied()
            else {
                return;
            };
            set_remember_mode(mode, &state_for_row, &ui_for_row);
        });
    } else {
        row.set_sensitive(false);
        row.set_tooltip_text(Some(&tr("This preference is managed by the system.")));
    }

    row
}

fn remember_mode_index(mode: RememberMode) -> u32 {
    RememberMode::SETTINGS_VALUES
        .iter()
        .position(|candidate| *candidate == mode)
        .unwrap_or(2) as u32
}

fn preference_group(
    title: &str,
    description: &str,
    rows: &[PreferenceSpec<'_>],
    advanced_features: bool,
    smart_insights_enabled: bool,
    preferences: &Preferences,
) -> Option<(adw::PreferencesGroup, SearchablePreferencesGroup)> {
    let group = adw::PreferencesGroup::builder()
        .title(tr(title))
        .description(tr(description))
        .build();
    let mut search_group = SearchablePreferencesGroup::new(&group, title, description);
    let mut added = false;
    for spec in rows {
        let writable = Preferences::key_for_action(spec.action_name)
            .map(|key| preferences.is_writable(key))
            .unwrap_or(true);
        if !spec.visible(writable, advanced_features, smart_insights_enabled) {
            continue;
        }
        let row = preference_row(spec, writable, smart_insights_enabled);
        search_group.add_row(&row, spec.title, spec.subtitle);
        group.add(&row);
        added = true;
    }
    added.then_some((group, search_group))
}

fn preference_row_visible(
    writable: bool,
    advanced_features: bool,
    smart_insights_enabled: bool,
    requires_smart_insights: bool,
) -> bool {
    (writable || advanced_features)
        && (!requires_smart_insights || smart_insights_enabled || advanced_features)
}

fn preference_row_enabled(
    writable: bool,
    smart_insights_enabled: bool,
    requires_smart_insights: bool,
) -> bool {
    writable && (!requires_smart_insights || smart_insights_enabled)
}

fn preference_row(
    spec: &PreferenceSpec<'_>,
    writable: bool,
    smart_insights_enabled: bool,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(tr(spec.title))
        .subtitle(tr(spec.subtitle))
        .build();
    row.set_active(spec.active);
    if spec.enabled(writable, smart_insights_enabled) {
        row.set_action_name(Some(spec.action_name));
    } else {
        row.set_sensitive(false);
        let message = if !writable {
            "This preference is managed by the system."
        } else {
            "Enable Smart Insights to use this preference."
        };
        row.set_tooltip_text(Some(&tr(message)));
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preference_row_visibility_follows_managed_state_and_mode() {
        assert!(preference_row_visible(true, false, false, false));
        assert!(preference_row_visible(true, true, false, false));
        assert!(!preference_row_visible(false, false, false, false));
        assert!(preference_row_visible(false, true, false, false));
    }

    #[test]
    fn smart_dependent_preferences_hide_in_simple_mode_and_show_in_advanced_mode() {
        assert!(preference_row_visible(true, false, true, true));
        assert!(!preference_row_visible(true, false, false, true));
        assert!(preference_row_visible(true, true, false, true));
    }

    #[test]
    fn smart_dependent_preferences_disable_without_smart_insights() {
        assert!(preference_row_enabled(true, true, true));
        assert!(!preference_row_enabled(true, false, true));
        assert!(!preference_row_enabled(false, true, true));
    }
}
