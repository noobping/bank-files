use super::*;
use chrono::NaiveDate;
use rust_decimal::Decimal;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn editable_rules_roundtrip_simple_and_regex_patterns() {
    let rules = vec![
        EditableRule {
            search: "Albert Heijn".to_string(),
            is_regex: false,
            ..EditableRule::new_default()
        },
        EditableRule {
            priority: 90,
            search: "(?i)github|openai".to_string(),
            is_regex: true,
            category: "Software".to_string(),
            budget_code: "CLOUD".to_string(),
            ..EditableRule::new_default()
        },
    ];

    let csv = serialize_editable_rules(&rules).unwrap();
    let parsed = parse_editable_rules(&csv).unwrap();

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].search, "Albert Heijn");
    assert!(!parsed[0].is_regex);
    assert_eq!(parsed[1].search, "(?i)github|openai");
    assert!(parsed[1].is_regex);
}

#[test]
fn simple_text_patterns_are_regex_escaped_on_save() {
    let rule = EditableRule {
        search: "C++ winkel?".to_string(),
        is_regex: false,
        ..EditableRule::new_default()
    };

    let csv = serialize_editable_rules(&[rule]).unwrap();

    assert!(csv.contains("C\\+\\+ winkel\\?"));
}

#[test]
fn group_rules_for_combining_moves_compatible_rules_together() {
    let groceries_alpha = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let groceries_beta = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[groceries_alpha, software, groceries_beta]);

    assert!(report.changed);
    assert_eq!(report.grouped_groups, 1);
    assert_eq!(report.rules[0].search, "alpha");
    assert_eq!(report.rules[1].search, "beta");
    assert_eq!(report.rules[2].search, "hosting");
    assert_eq!(combine_editable_rules(&report.rules).after_count, 2);
}

#[test]
fn group_rules_for_combining_detects_already_grouped_rules() {
    let first = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[first, second]);

    assert!(!report.changed);
    assert_eq!(report.grouped_groups, 1);
}

#[test]
fn group_rules_for_combining_ignores_singletons() {
    let groceries = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };

    let report = group_editable_rules_for_combining(&[groceries, software]);

    assert!(!report.changed);
    assert_eq!(report.grouped_groups, 0);
}

#[test]
fn combine_rules_merges_adjacent_plain_rules() {
    let first = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.before_count, 2);
    assert_eq!(report.after_count, 1);
    assert_eq!(report.combined_groups, 1);
    assert!(report.rules[0].is_regex);
    assert_eq!(report.rules[0].search, "(?:alpha|beta)");
}

#[test]
fn combine_rules_expands_existing_clean_regex() {
    let regex_rule = EditableRule {
        search: "(?:alpha|beta)".to_string(),
        is_regex: true,
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let plain_rule = EditableRule {
        search: "gamma".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[regex_rule, plain_rule]);

    assert_eq!(report.after_count, 1);
    assert_eq!(report.rules[0].search, "(?:alpha|beta|gamma)");
}

#[test]
fn combine_rules_builds_literal_regex_with_flexible_spaces_and_shared_prefix() {
    let first = EditableRule {
        search: "Albert  Heijn Amsterdam".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: "Albert Heijn Utrecht".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.after_count, 1);
    assert_eq!(
        report.rules[0].search,
        r"Albert\s+Heijn(?:\s+Amsterdam|\s+Utrecht)"
    );
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_merges_existing_regex_terms() {
    let first = EditableRule {
        search: r"(?:amazon(?: marketplace)?|bol\.com)".to_string(),
        is_regex: true,
        category: "Shopping".to_string(),
        budget_code: "SHOP".to_string(),
        ..EditableRule::new_default()
    };
    let second = EditableRule {
        search: r"(?:bol\.com|coolblue)".to_string(),
        is_regex: true,
        category: "Shopping".to_string(),
        budget_code: "SHOP".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[first, second]);

    assert_eq!(report.after_count, 1);
    assert_eq!(
        report.rules[0].search,
        r"(?:amazon(?: marketplace)?|bol\.com|coolblue)"
    );
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_merges_case_insensitive_regex_wrappers() {
    let regex_rule = EditableRule {
        search: "(?i:github|openai)".to_string(),
        is_regex: true,
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let plain_rule = EditableRule {
        search: "Anthropic".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[regex_rule, plain_rule]);

    assert_eq!(report.after_count, 1);
    assert_eq!(report.rules[0].search, "(?:github|openai|Anthropic)");
    assert!(report.rules[0].is_regex);
}

#[test]
fn combine_rules_keeps_non_adjacent_rules_in_place() {
    let groceries_alpha = EditableRule {
        search: "alpha".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };
    let software = EditableRule {
        search: "hosting".to_string(),
        category: "Software".to_string(),
        budget_code: "CLOUD".to_string(),
        ..EditableRule::new_default()
    };
    let groceries_beta = EditableRule {
        search: "beta".to_string(),
        category: "Groceries".to_string(),
        budget_code: "FOOD".to_string(),
        ..EditableRule::new_default()
    };

    let report = combine_editable_rules(&[groceries_alpha, software, groceries_beta]);

    assert_eq!(report.after_count, 3);
    assert_eq!(report.combined_groups, 0);
}

#[test]
fn dedupe_enabled_removes_duplicates_and_disabled_keeps_them() {
    let transactions = vec![test_transaction("a"), test_transaction("b")];

    let (deduped, removed) = dedupe(transactions.clone(), DedupeMode::Enabled);
    assert_eq!(deduped.len(), 1);
    assert_eq!(removed, 1);

    let (kept, removed) = dedupe(transactions, DedupeMode::Disabled);
    assert_eq!(kept.len(), 2);
    assert_eq!(removed, 0);
}

#[test]
fn config_csv_names_map_to_config_files() {
    assert_eq!(
        config_csv_name(Path::new("budgetcodes.csv")),
        Some("budgetcodes.csv")
    );
    assert_eq!(
        config_csv_name(Path::new("budgets.csv")),
        Some("budgetcodes.csv")
    );
    assert_eq!(config_csv_name(Path::new("rules.csv")), Some("rules.csv"));
    assert_eq!(
        config_csv_name(Path::new("field-aliases.csv")),
        Some("field_aliases.csv")
    );
    assert_eq!(config_csv_name(Path::new("transactions.csv")), None);
}

#[test]
fn config_csv_headers_map_to_config_files() {
    assert_eq!(
        config_csv_from_headers(
            "code,category,monthly_budget,yearly_budget,direction,notes\nFOOD,Groceries,500,\n"
        ),
        Some("budgetcodes.csv")
    );
    assert_eq!(
            config_csv_from_headers(
                "priority;active;field;pattern;category;budget_code;direction\n100;true;any;test;Other;OTHER;expense\n"
            ),
            Some("rules.csv")
        );
    assert_eq!(
        config_csv_from_headers("canonical\talias\ndate\tDatum\n"),
        Some("field_aliases.csv")
    );
    assert_eq!(
        config_csv_from_headers("Date,Description,Amount\n2026-01-01,Coffee,-2.50\n"),
        None
    );
}

#[test]
fn editable_budgets_accept_income_percentages() {
    let budgets = vec![EditableBudget {
        code: "SAVE".to_string(),
        category: "Savings".to_string(),
        monthly_budget: "10%".to_string(),
        yearly_budget: String::new(),
        direction: "expense".to_string(),
        income_basis: "planned".to_string(),
        notes: "Income based".to_string(),
    }];

    validate_editable_budgets(&budgets).unwrap();
    let csv = serialize_editable_budgets(&budgets).unwrap();
    let parsed = parse_editable_budgets(&csv).unwrap();

    assert_eq!(parsed[0].monthly_budget, "10%");
    assert_eq!(parsed[0].yearly_budget, "");
    assert_eq!(parsed[0].income_basis, "planned");
}

#[test]
fn upsert_alias_adds_once() {
    let mut aliases = Vec::new();
    assert!(super::config::upsert_alias(&mut aliases, "date", "Boekdatum").unwrap());
    assert!(!super::config::upsert_alias(&mut aliases, "date", "boekdatum").unwrap());

    assert_eq!(
        aliases
            .iter()
            .filter(
                |alias| alias.canonical == "date" && alias.alias.eq_ignore_ascii_case("boekdatum")
            )
            .count(),
        1
    );
}

#[test]
fn copy_gio_files_routes_transactions_and_config_csvs() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-csv-routing-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    ensure_layout(&dirs).expect("create app dirs");

    let transaction = root.join("transactions.csv");
    fs::write(
        &transaction,
        "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n",
    )
    .expect("write transaction csv");
    let budget = root.join("monthly-plan.csv");
    fs::write(
        &budget,
        "code,category,monthly_budget,yearly_budget,direction,notes\nFOOD,Groceries,500,\n",
    )
    .expect("write budget csv");

    assert_eq!(
        copy_gio_file_to_app_storage(&adw::gtk::gio::File::for_path(&transaction), &dirs)
            .expect("copy transaction"),
        CsvCopyTarget::Transactions
    );
    assert_eq!(
        copy_gio_file_to_app_storage(&adw::gtk::gio::File::for_path(&budget), &dirs)
            .expect("copy budget"),
        CsvCopyTarget::Config
    );
    let copied_transaction = dirs.inbox.join("transactions.csv");
    assert!(copied_transaction.exists());
    assert!(dirs.config.join("budgetcodes.csv").exists());
    assert!(fs::metadata(&copied_transaction)
        .expect("copied transaction metadata")
        .permissions()
        .readonly());
    let mut permissions = fs::metadata(&copied_transaction)
        .expect("copied transaction metadata")
        .permissions();
    #[cfg(unix)]
    permissions.set_mode(permissions.mode() | 0o200);
    #[cfg(not(unix))]
    permissions.set_readonly(false);
    let _ = fs::set_permissions(&copied_transaction, permissions);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn legacy_inbox_csvs_move_to_app_data_folder() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-layout-migration-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    let legacy_inbox = dirs.data.join("inbox");
    fs::create_dir_all(&legacy_inbox).expect("create legacy inbox");
    fs::write(
        legacy_inbox.join("transactions.csv"),
        "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n",
    )
    .expect("write legacy csv");

    ensure_layout(&dirs).expect("create app dirs");
    migrate_legacy_app_data_layout(&dirs).expect("migrate layout");

    assert!(dirs.data.join("transactions.csv").exists());
    assert!(!legacy_inbox.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn configuration_archive_replaces_existing_archive() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-archive-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, "rules-v1", "budgets-v1", "aliases-v1");

    let archive = config::archive_configuration_in(&dirs).expect("archive configuration");
    fs::write(archive.join("stale.txt"), "stale").expect("write stale archive file");
    write_test_configuration(&dirs, "rules-v2", "budgets-v2", "aliases-v2");

    let archive = config::archive_configuration_in(&dirs).expect("replace archive");

    assert_eq!(
        fs::read_to_string(archive.join("rules.csv")).unwrap(),
        "rules-v2"
    );
    assert_eq!(
        fs::read_to_string(archive.join("budgetcodes.csv")).unwrap(),
        "budgets-v2"
    );
    assert_eq!(
        fs::read_to_string(archive.join("field_aliases.csv")).unwrap(),
        "aliases-v2"
    );
    assert!(!archive.join("stale.txt").exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn configuration_archive_restore_requires_archive_and_restores_files() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-restore-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, "rules-old", "budgets-old", "aliases-old");

    assert!(config::restore_configuration_archive_in(&dirs).is_err());
    config::archive_configuration_in(&dirs).expect("archive configuration");
    write_test_configuration(&dirs, "rules-new", "budgets-new", "aliases-new");

    config::restore_configuration_archive_in(&dirs).expect("restore configuration");

    assert_eq!(
        fs::read_to_string(dirs.config.join("rules.csv")).unwrap(),
        "rules-old"
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("budgetcodes.csv")).unwrap(),
        "budgets-old"
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("field_aliases.csv")).unwrap(),
        "aliases-old"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn restore_default_configuration_replaces_all_config_files() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-defaults-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, "custom-rules", "custom-budgets", "custom-aliases");

    config::restore_default_configuration_in(&dirs).expect("restore default configuration");

    assert_eq!(
        fs::read_to_string(dirs.config.join("rules.csv")).unwrap(),
        default_rules()
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("budgetcodes.csv")).unwrap(),
        default_budgets()
    );
    assert_eq!(
        fs::read_to_string(dirs.config.join("field_aliases.csv")).unwrap(),
        default_aliases()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn restore_empty_configuration_clears_rules_and_budgets() {
    let unique = chrono::Local::now().format("%Y%m%d%H%M%S%f");
    let root = std::env::temp_dir().join(format!("bank-files-config-empty-{unique}"));
    let dirs = AppDirs {
        config: root.join("config"),
        data: root.join("data"),
        inbox: root.join("data"),
    };
    write_test_configuration(&dirs, default_rules(), default_budgets(), "custom-aliases");

    config::restore_empty_configuration_in(&dirs).expect("restore empty configuration");

    let rules = fs::read_to_string(dirs.config.join("rules.csv")).unwrap();
    let budgets = fs::read_to_string(dirs.config.join("budgetcodes.csv")).unwrap();
    assert!(parse_editable_rules(&rules).unwrap().is_empty());
    assert!(parse_editable_budgets(&budgets).unwrap().is_empty());
    assert_eq!(
        fs::read_to_string(dirs.config.join("field_aliases.csv")).unwrap(),
        default_aliases()
    );

    let _ = fs::remove_dir_all(root);
}

fn write_test_configuration(dirs: &AppDirs, rules: &str, budgets: &str, aliases: &str) {
    ensure_layout(dirs).expect("create app dirs");
    fs::write(dirs.config.join("rules.csv"), rules).expect("write rules");
    fs::write(dirs.config.join("budgetcodes.csv"), budgets).expect("write budgets");
    fs::write(dirs.config.join("field_aliases.csv"), aliases).expect("write aliases");
}

fn test_transaction(id: &str) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        amount: Decimal::new(-1234, 2),
        description: "Coffee".to_string(),
        tags: String::new(),
        counterparty: "Cafe".to_string(),
        account: "NL00TEST".to_string(),
        transaction_id: id.to_string(),
        currency: "EUR".to_string(),
        source_file: "test.csv".to_string(),
        source_row: 1,
        category: "Dining out".to_string(),
        budget_code: "HORECA".to_string(),
        notes: String::new(),
        strict_key: "same-strict-key".to_string(),
        loose_key: format!("loose-{id}"),
    }
}
