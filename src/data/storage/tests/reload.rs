use super::*;

#[test]
fn reload_inbox_file_replaces_only_selected_file() {
    let root = unique_test_dir("reload-inbox-file");
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    ensure_layout(&dirs).expect("test app dirs should be created");
    ensure_default_files(&dirs).expect("test config should be created");

    let selected_csv = dirs.inbox.join("selected.csv");
    let other_csv = dirs.inbox.join("other.csv");
    fs::write(
        &selected_csv,
        "Date,Description,Amount\n2026-01-03,Fresh selected,-12.34\n",
    )
    .expect("selected test csv should be written");
    fs::write(
        &other_csv,
        "Date,Description,Amount\n2026-01-04,Fresh other,-8.00\n",
    )
    .expect("other test csv should be written");
    mark_transaction_csv_readonly(&selected_csv).expect("selected csv should be read-only");

    let old_warning = format!("{}: old selected warning", selected_csv.display());
    let other_warning = format!("{}: keep other warning", other_csv.display());
    let data = AppData {
        transactions: vec![
            transaction("selected.csv", "Old selected"),
            transaction("other.csv", "Keep other"),
        ],
        reports: vec![import_report(&selected_csv), import_report(&other_csv)],
        warnings: vec![old_warning.clone(), other_warning.clone()],
        dedupe_mode: DedupeMode::Disabled,
        available_months: vec![MonthKey::new(2026, 1)],
        available_years: vec![2026],
        default_month: Some(MonthKey::new(2026, 1)),
        loaded_scope: TransactionLoadScope::All,
        ..AppData::default()
    };

    let reloaded = reload_inbox_file_with_dirs(data, &dirs, &selected_csv, DedupeMode::Disabled)
        .expect("selected csv should reload");

    assert!(reloaded
        .transactions
        .iter()
        .any(|transaction| transaction.source_file == "selected.csv"
            && transaction.description == "Fresh selected"));
    assert!(!reloaded
        .transactions
        .iter()
        .any(|transaction| transaction.source_file == "selected.csv"
            && transaction.description == "Old selected"));
    assert!(reloaded
        .transactions
        .iter()
        .any(|transaction| transaction.source_file == "other.csv"
            && transaction.description == "Keep other"));
    assert!(reloaded
        .reports
        .iter()
        .any(|report| report.source == selected_csv && report.rows_imported == 1));
    assert!(!reloaded.warnings.contains(&old_warning));
    assert!(reloaded.warnings.contains(&other_warning));

    fs::remove_dir_all(root).ok();
}
