use super::*;

#[test]
fn storage_capabilities_report_readonly_data_folder() {
    let root = unique_test_dir("readonly-data-capability");
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    ensure_layout(&dirs).expect("test app dirs should be created");
    let mut permissions = fs::metadata(&dirs.data)
        .expect("data metadata should exist")
        .permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&dirs.data, permissions).expect("data folder should become read-only");

    let capabilities = crate::data::storage_capabilities(&dirs);

    assert!(!capabilities.data_writable);
    assert!(capabilities.config_writable);
    assert!(capabilities.data_write_reason().contains("read-only"));

    let mut permissions = fs::metadata(&dirs.data)
        .expect("data metadata should exist")
        .permissions();
    make_permissions_writable(&mut permissions);
    let _ = fs::set_permissions(&dirs.data, permissions);
    fs::remove_dir_all(root).ok();
}

#[test]
fn mark_transaction_csv_readonly_sets_readonly_permission() {
    let root = unique_test_dir("mark-csv-readonly");
    fs::create_dir_all(&root).expect("test root should be created");
    let csv = root.join("transactions.csv");
    fs::write(
        &csv,
        "Date,Description,Amount
2026-01-01,Coffee,-2.50
",
    )
    .expect("test csv should be written");

    mark_transaction_csv_readonly(&csv).expect("csv should be marked read-only");

    assert!(fs::metadata(&csv)
        .expect("csv metadata should exist")
        .permissions()
        .readonly());

    let mut permissions = fs::metadata(&csv)
        .expect("csv metadata should exist")
        .permissions();
    make_permissions_writable(&mut permissions);
    let _ = fs::set_permissions(&csv, permissions);
    fs::remove_dir_all(root).ok();
}
