use super::group::preference_row_visible;
use super::*;

pub(super) fn preferences_page_snapshot(
    advanced_features: bool,
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
            ),
            (
                "Always Show Full Lists",
                "Show every item immediately and hide More rows.",
                "app.show-all",
            ),
            (
                "Compare Spending with Previous Period",
                "Compare spending cards with the previous month or year.",
                "app.compare-categories-previous-period",
            ),
        ],
        advanced_features,
        preferences,
    );
    add_preference_snapshot_rows(
        &mut rows,
        "Remember",
        &[(
            "Remember",
            "Choose whether opened bank files are forgotten after this session, remembered as data, or remembered with reusable analysis cache.",
            "app.remember-mode",
        )],
        advanced_features,
        preferences,
    );
    add_preference_snapshot_rows(
        &mut rows,
        "Forms and Data",
        &[
            (
                "Advanced Features",
                "Show budget codes, direction controls, and detailed move options.",
                "app.advanced-features",
            ),
            (
                "Whole Form Autofill",
                "Fill related form fields from the value you choose, such as matching categories, budget codes, and directions.",
                "app.advanced-autofill",
            ),
            (
                "Hide Refunded Transactions",
                "Hide REFUNDING and REFUNDED transactions from normal views and totals.",
                "app.hide-refunded-transactions",
            ),
        ],
        advanced_features,
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
    specs: &[(&str, &str, &str)],
    advanced_features: bool,
    preferences: &Preferences,
) {
    for (title, subtitle, action_name) in specs {
        let writable = Preferences::key_for_action(action_name)
            .map(|key| preferences.is_writable(key))
            .unwrap_or(true);
        if preference_row_visible(writable, advanced_features) {
            rows.push(vec![tr(group_title), tr(title), tr(subtitle)]);
        }
    }
}
