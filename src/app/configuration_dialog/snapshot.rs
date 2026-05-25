use super::*;

pub(super) fn configuration_page_snapshot() -> StaticPageSnapshot {
    StaticPageSnapshot::new(
        "configuration",
        "Configuration",
        "Configuration actions report progress here.",
        &["Group", "Action", "Description"],
        configuration_snapshot_rows(),
    )
}

pub(super) fn configuration_snapshot_rows() -> Vec<Vec<String>> {
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

    rows
}
