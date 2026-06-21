use super::*;

#[test]
fn read_only_load_uses_defaults_without_creating_missing_config() {
    let root = unique_test_dir("read-only-load-defaults");
    fs::create_dir_all(&root).expect("test root should be created");
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    let capabilities = StorageCapabilities {
        data_readable: true,
        data_writable: false,
        config_readable: true,
        config_writable: false,
        data_reason: "Bank file storage is read-only.".to_string(),
        config_reason: "Configuration storage is read-only.".to_string(),
    };

    let (data, returned_capabilities) = load_app_data_from_dirs(
        &dirs,
        &capabilities,
        AppDataLoadRequest {
            mode: DedupeMode::Disabled,
            scope: TransactionLoadScope::All,
            remember_mode: RememberMode::DataOnly,
            sources: &[],
        },
    )
    .expect("read-only load should use embedded defaults");

    assert!(data.transactions.is_empty());
    assert!(!dirs.config.exists());
    assert!(!dirs.inbox.exists());
    assert!(!data.budgets.is_empty());
    assert_eq!(returned_capabilities, capabilities);

    fs::remove_dir_all(root).ok();
}

#[test]
fn forget_mode_loads_live_csv_without_creating_app_storage() {
    let root = unique_test_dir("forget-mode-live-csv");
    fs::create_dir_all(&root).expect("test root should be created");
    let live_csv = root.join("live.csv");
    fs::write(
        &live_csv,
        "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n",
    )
    .expect("live test csv should be written");
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    let capabilities = StorageCapabilities {
        data_readable: true,
        data_writable: false,
        config_readable: true,
        config_writable: false,
        data_reason: "Bank file storage is read-only.".to_string(),
        config_reason: "Configuration storage is read-only.".to_string(),
    };
    let sources = vec![TransactionSource::live_file(live_csv.clone())];

    let (data, returned_capabilities) = load_app_data_from_dirs(
        &dirs,
        &capabilities,
        AppDataLoadRequest {
            mode: DedupeMode::Disabled,
            scope: TransactionLoadScope::All,
            remember_mode: RememberMode::Forget,
            sources: &sources,
        },
    )
    .expect("live load should read the selected CSV");

    assert_eq!(data.transactions.len(), 1);
    assert_eq!(data.remember_mode, RememberMode::Forget);
    assert_eq!(data.transaction_sources, sources);
    assert!(!dirs.config.exists());
    assert!(!dirs.inbox.exists());
    assert_eq!(returned_capabilities, capabilities);

    fs::remove_dir_all(root).ok();
}
