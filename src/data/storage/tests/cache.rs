use super::*;

#[test]
fn clear_processed_app_data_cache_removes_only_processed_cache() {
    let _guard = CACHE_ENV_LOCK
        .lock()
        .expect("cache env lock should be available");
    let root = unique_test_dir("clear-processed-cache");
    fs::create_dir_all(&root).expect("test root should be created");
    std::env::set_var("BANK_FILES_CACHE", root.join("cache"));
    let processed = root.join("cache").join("processed");
    let sibling = root.join("cache").join("other");
    fs::create_dir_all(&processed).expect("processed cache should be created");
    fs::create_dir_all(&sibling).expect("sibling cache should be created");
    fs::write(processed.join("entry.json"), "{}").expect("cache file should be written");
    fs::write(sibling.join("keep.txt"), "keep").expect("sibling file should be written");

    assert!(clear_processed_app_data_cache().expect("cache cleanup should succeed"));
    assert!(!processed.exists());
    assert!(sibling.exists());
    assert!(!clear_processed_app_data_cache().expect("missing cache is ok"));

    std::env::remove_var("BANK_FILES_CACHE");
    fs::remove_dir_all(root).ok();
}

#[test]
fn data_and_analytics_cache_reuses_processed_live_data() {
    let _guard = CACHE_ENV_LOCK
        .lock()
        .expect("cache env lock should be available");
    let root = unique_test_dir("full-cache-live-csv");
    fs::create_dir_all(&root).expect("test root should be created");
    std::env::set_var("BANK_FILES_CACHE", root.join("cache"));
    let live_csv = root.join("live.csv");
    fs::write(
        &live_csv,
        "Date,Description,Amount\n2026-01-01,Tikkie dinner,-2.50\n",
    )
    .expect("live test csv should be written");
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    let capabilities = StorageCapabilities::default();
    let sources = vec![TransactionSource::live_file(live_csv)];

    let (first, _) = load_app_data_from_dirs(
        &dirs,
        &capabilities,
        AppDataLoadRequest {
            mode: DedupeMode::Disabled,
            auto_clean_config: false,
            scope: TransactionLoadScope::All,
            remember_mode: RememberMode::DataAndAnalytics,
            sources: &sources,
        },
    )
    .expect("first full-cache load should parse and cache data");
    let (second, _) = load_app_data_from_dirs(
        &dirs,
        &capabilities,
        AppDataLoadRequest {
            mode: DedupeMode::Disabled,
            auto_clean_config: false,
            scope: TransactionLoadScope::All,
            remember_mode: RememberMode::DataAndAnalytics,
            sources: &sources,
        },
    )
    .expect("second full-cache load should reuse cached data");
    assert_eq!(first.cache_status, DataCacheStatus::Updated);
    assert_eq!(second.cache_status, DataCacheStatus::Hit);
    assert_eq!(second.transactions.len(), 1);
    assert_eq!(first.transactions[0].budget_code, "OTHER");

    std::env::remove_var("BANK_FILES_CACHE");
    fs::remove_dir_all(root).ok();
}
