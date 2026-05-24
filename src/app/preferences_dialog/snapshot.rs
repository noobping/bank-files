use super::group::preference_row_visible;
use super::*;

pub(super) fn preferences_page_snapshot(
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
            (
                "Compare Spending with Previous Period",
                "Compare spending cards with the previous month or year.",
                "app.compare-categories-previous-period",
                false,
            ),
        ],
        advanced_features,
        smart_insights_enabled,
        preferences,
    );
    let experimental_rows = Vec::new();
    #[cfg(feature = "smart-insights")]
    let mut experimental_rows = experimental_rows;
    #[cfg(feature = "smart-insights")]
    {
        experimental_rows.push((
            "Smart Insights",
            "Show forecast cards and detect transaction patterns, including possible transfers, from imported transactions.",
            "app.show-predictions",
            false,
        ));
        #[cfg(not(feature = "flatpak"))]
        experimental_rows.push((
            "Online Smart Insights",
            "Allow privacy-filtered company category lookups. Amounts, dates, accounts, descriptions, notes, and rows are never sent.",
            "app.online-smart-insights",
            true,
        ));
        experimental_rows.push((
            "Hide Refunded Transactions",
            "Requires Smart Insights. Exclude detected refunds and offsetting groups from normal views.",
            "app.hide-canceled-transactions",
            true,
        ));
    }
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
    let forms_rows = vec![
        (
            "Advanced Features",
            "Allow rule editing and budget direction controls.",
            "app.advanced-features",
            false,
        ),
        (
            "Whole Form Autofill",
            "Fill related form fields from the value you choose, such as matching categories, budget codes, and directions.",
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
    add_preference_snapshot_rows(
        &mut rows,
        "Forms and Data",
        &forms_rows,
        advanced_features,
        smart_insights_enabled,
        preferences,
    );

    if advanced_features {
        add_preference_snapshot_rows(
            &mut rows,
            "Experimental",
            &experimental_rows,
            advanced_features,
            smart_insights_enabled,
            preferences,
        );
    }

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
