use super::{can_install_into, install_target_dir_is_eligible, is_writable};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn writability_probe_does_not_truncate_existing_perm_test_files() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("bank-files-setup-writable-{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let existing = dir.join(".perm_test");
    fs::write(&existing, "keep").expect("write marker");

    assert!(is_writable(&dir));
    assert_eq!(
        fs::read_to_string(&existing).expect("read marker"),
        "keep".to_string()
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn install_target_dir_rejects_existing_non_writable_directories() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("bank-files-setup-existing-dir-{unique}"));
    let target = root.join("applications");
    fs::create_dir_all(&target).expect("create target dir");
    let mut permissions = fs::metadata(&target)
        .expect("read target metadata")
        .permissions();
    permissions.set_mode(0o500);
    fs::set_permissions(&target, permissions).expect("make target non-writable");

    assert!(!install_target_dir_is_eligible(&target));

    let mut cleanup_permissions = fs::metadata(&target)
        .expect("read target metadata for cleanup")
        .permissions();
    cleanup_permissions.set_mode(0o700);
    fs::set_permissions(&target, cleanup_permissions).expect("restore target permissions");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn install_eligibility_checks_all_written_target_directories() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("bank-files-setup-eligibility-{unique}"));
    let bin = root.join("bin");
    let data = root.join("data");
    let icons = data
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps");
    fs::create_dir_all(&bin).expect("create bin dir");
    fs::create_dir_all(&icons).expect("create icons dir");
    let mut permissions = fs::metadata(&icons)
        .expect("read icon metadata")
        .permissions();
    permissions.set_mode(0o500);
    fs::set_permissions(&icons, permissions).expect("make icons non-writable");

    assert!(!can_install_into(&bin, &data));

    let mut cleanup_permissions = fs::metadata(&icons)
        .expect("read icon metadata for cleanup")
        .permissions();
    cleanup_permissions.set_mode(0o700);
    fs::set_permissions(&icons, cleanup_permissions).expect("restore icon permissions");
    let _ = fs::remove_dir_all(root);
}
