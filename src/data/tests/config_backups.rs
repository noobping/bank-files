use super::*;

#[test]
fn configuration_archive_keeps_latest_five_archives() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-archive-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };

    for index in 0..6 {
        write_test_configuration(
            &dirs,
            &format!("rules-v{index}"),
            &format!("budgets-v{index}"),
            &format!("aliases-v{index}"),
        );
        config::archive_configuration_in(&dirs).expect("archive configuration");
    }

    let archives = config::configuration_archives_in(&dirs).expect("list archives");

    assert_eq!(archives.len(), 5);
    assert!(archives
        .iter()
        .all(|archive| archive.path.join("rules.csv").is_file()));
    assert!(archives
        .iter()
        .all(|archive| archive.path.join("budgetcodes.csv").is_file()));
    assert!(archives
        .iter()
        .all(|archive| archive.path.join("field_aliases.csv").is_file()));
    assert!(!archives
        .iter()
        .any(|archive| fs::read_to_string(archive.path.join("rules.csv")).unwrap() == "rules-v0"));

    write_test_configuration(&dirs, "rules-current", "budgets-current", "aliases-current");
    config::restore_configuration_archive_in(&dirs).expect("restore latest archive");

    assert_eq!(
        fs::read_to_string(dirs.config.join("rules.csv")).unwrap(),
        "rules-v5"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn configuration_archive_restore_requires_archive_and_restores_latest_files() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-restore-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, "rules-old", "budgets-old", "aliases-old");

    assert!(config::restore_configuration_archive_in(&dirs).is_err());
    config::archive_configuration_in(&dirs).expect("archive old configuration");
    write_test_configuration(&dirs, "rules-new", "budgets-new", "aliases-new");
    config::archive_configuration_in(&dirs).expect("archive new configuration");
    write_test_configuration(&dirs, "rules-current", "budgets-current", "aliases-current");

    config::restore_configuration_archive_in(&dirs).expect("restore latest configuration");

    assert_eq!(
        fs::read_to_string(dirs.config.join("rules.csv")).unwrap(),
        "rules-new"
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("budgetcodes.csv")).unwrap(),
        "budgets-new"
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("field_aliases.csv")).unwrap(),
        "aliases-new"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn configuration_archive_can_restore_and_remove_selected_archive() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-select-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, "rules-first", "budgets-first", "aliases-first");
    let first_path = config::archive_configuration_in(&dirs).expect("archive first");
    let first_id = archive_id(&first_path);
    write_test_configuration(&dirs, "rules-second", "budgets-second", "aliases-second");
    let second_path = config::archive_configuration_in(&dirs).expect("archive second");
    let second_id = archive_id(&second_path);
    write_test_configuration(&dirs, "rules-current", "budgets-current", "aliases-current");

    config::restore_configuration_archive_by_id_in(&dirs, &first_id)
        .expect("restore selected archive");

    assert_eq!(
        fs::read_to_string(dirs.config.join("rules.csv")).unwrap(),
        "rules-first"
    );

    config::remove_configuration_archive_in(&dirs, &second_id).expect("remove selected archive");
    let archives = config::configuration_archives_in(&dirs).expect("list archives");

    assert!(archives.iter().any(|archive| archive.id == first_id));
    assert!(!archives.iter().any(|archive| archive.id == second_id));

    let _ = fs::remove_dir_all(root);
}

fn write_test_configuration(dirs: &AppDirs, rules: &str, budgets: &str, aliases: &str) {
    ensure_layout(dirs).expect("create app dirs");
    fs::write(dirs.config.join("rules.csv"), rules).expect("write rules");
    fs::write(dirs.config.join("budgetcodes.csv"), budgets).expect("write budgets");
    fs::write(dirs.config.join("field_aliases.csv"), aliases).expect("write aliases");
}

fn archive_id(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .expect("archive id")
        .to_string()
}
