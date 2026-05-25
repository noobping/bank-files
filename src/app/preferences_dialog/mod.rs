use super::*;

mod group;
mod remember;
mod snapshot;
mod spec;

use group::preference_group;
use remember::remember_preference_group;
use snapshot::preferences_page_snapshot;
use spec::PreferenceSpec;

pub(in crate::app) fn show_preferences_dialog(
    parent: &adw::ApplicationWindow,
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
) {
    let shell = build_settings_dialog_shell("Preferences", "Search preferences");
    let root = shell.root;
    let search_bar = shell.search_bar;
    let search_entry = shell.search_entry;

    let page = adw::PreferencesPage::builder()
        .title(tr("General"))
        .icon_name("preferences-system-symbolic")
        .build();
    let mut search_groups = Vec::new();
    let advanced_features = ui.advanced_features.get();
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
                "Show every item immediately and hide More rows.",
                "app.show-all",
                ui.show_all.get(),
            ),
            PreferenceSpec::new(
                "Compare Spending with Previous Period",
                "Compare spending cards with the previous month or year.",
                "app.compare-categories-previous-period",
                ui.compare_categories_previous_period.get(),
            ),
        ],
        advanced_features,
        smart_insights_enabled,
        &ui.preferences,
    ) {
        page.add(&group);
        search_groups.push(search_group);
    }

    let experimental_preferences = Vec::new();
    #[cfg(feature = "smart-insights")]
    let mut experimental_preferences = experimental_preferences;
    #[cfg(feature = "smart-insights")]
    {
        let smart_dependent_preferences_enabled = Rc::new(Cell::new(smart_insights_enabled));
        experimental_preferences.push(
            PreferenceSpec::new(
                "Smart Insights",
                "Show forecast cards and detect transaction patterns, including possible transfers, from imported transactions.",
                "app.show-predictions",
                ui.show_predictions.get(),
            )
            .toggles_enabled(Rc::clone(&smart_dependent_preferences_enabled)),
        );
        #[cfg(not(feature = "flatpak"))]
        experimental_preferences.push(
            PreferenceSpec::new(
                "Online Smart Insights",
                "Allow privacy-filtered company category lookups. Amounts, dates, accounts, descriptions, notes, and rows are never sent.",
                "app.online-smart-insights",
                ui.online_smart_insights.get(),
            )
            .requires_smart_insights()
            .enabled_by(Rc::clone(&smart_dependent_preferences_enabled)),
        );
        experimental_preferences.push(
            PreferenceSpec::new(
                "Hide Refunded Transactions",
                "Requires Smart Insights. Exclude detected refunds and offsetting groups from normal views.",
                "app.hide-canceled-transactions",
                ui.hide_canceled_transactions.get(),
            )
            .requires_smart_insights()
            .enabled_by(Rc::clone(&smart_dependent_preferences_enabled)),
        );
    }

    let advanced_preferences_visible = Rc::new(Cell::new(advanced_features));
    let mut experimental_group = preference_group(
        "Experimental",
        "Control Smart Insights, online enrichment, and detected refund hiding.",
        &experimental_preferences,
        true,
        smart_insights_enabled,
        &ui.preferences,
    );
    if let Some((group, search_group)) = &mut experimental_group {
        group.set_visible(advanced_features);
        search_group.set_visibility_gate(Rc::clone(&advanced_preferences_visible));
    }

    if let Some((group, search_group)) = remember_preference_group(advanced_features, state, ui) {
        page.add(&group);
        search_groups.push(search_group);
    }

    let mut advanced_features_spec = PreferenceSpec::new(
        "Advanced Features",
        "Allow rule editing and budget direction controls.",
        "app.advanced-features",
        advanced_features,
    );
    if let Some((group, _)) = &experimental_group {
        advanced_features_spec = advanced_features_spec
            .toggles_visibility(group, Rc::clone(&advanced_preferences_visible));
    }

    let forms_preferences = vec![
        advanced_features_spec,
        PreferenceSpec::new(
            "Whole Form Autofill",
            "Fill related form fields from the value you choose, such as matching categories, budget codes, and directions.",
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

    if let Some((group, search_group)) = preference_group(
        "Forms and Data",
        "Control simple mode, whole-form autofill, and cleanup.",
        &forms_preferences,
        advanced_features,
        smart_insights_enabled,
        &ui.preferences,
    ) {
        page.add(&group);
        search_groups.push(search_group);
    }

    if let Some((group, search_group)) = experimental_group {
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
        preferences_page_snapshot(advanced_features, smart_insights_enabled, &ui.preferences),
    );
    status_bar
        .label
        .set_text(&tr("Preference changes are applied immediately."));
    root.append(&status_bar.container);

    let dialog = ui::content_dialog(tr("Preferences"), &root)
        .content_width(680)
        .content_height(620)
        .build();

    ui::bind_search_bar(&dialog, &dialog, &search_bar, &search_entry);
    connect_preference_search(&search_entry, search_groups);

    dialog.present(Some(parent));
}

#[cfg(test)]
mod tests;
