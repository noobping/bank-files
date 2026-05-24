use super::groups::{
    automatic_configuration_generation_subtitle, automatic_configuration_generation_visible,
};
use super::*;

pub(super) fn configuration_page_snapshot(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> StaticPageSnapshot {
    StaticPageSnapshot::new(
        "configuration",
        "Configuration",
        "Configuration actions report progress here.",
        &["Group", "Action", "Description"],
        configuration_snapshot_rows(advanced_features, smart_insights_enabled),
    )
}

pub(super) fn configuration_snapshot_rows(
    advanced_features: bool,
    smart_insights_enabled: bool,
) -> Vec<Vec<String>> {
    let mut rows = vec![
        vec![
            tr("Configuration Backup"),
            tr("Back Up Current Configuration"),
            tr("Replace the existing backup in the config folder."),
        ],
        vec![
            tr("Configuration Backup"),
            tr("Restore Configuration Backup"),
            tr("Restore rules, budgets, and field names from the backup."),
        ],
    ];

    rows.extend([
        vec![
            tr("Configuration Templates"),
            tr("Use Default Configuration"),
            tr("Replace rules, budgets, and field names with the built-in defaults."),
        ],
        vec![
            tr("Configuration Templates"),
            tr("Use Empty Configuration"),
            tr("Remove all rules and budget codes while keeping CSV field names for imports."),
        ],
    ]);

    if automatic_configuration_generation_visible(advanced_features) {
        rows.push(vec![
            tr("Experimental"),
            tr("Generate Configuration from Transactions"),
            tr(automatic_configuration_generation_subtitle(
                smart_insights_enabled,
            )),
        ]);
    }

    rows
}
