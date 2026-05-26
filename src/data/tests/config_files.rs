use super::*;

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
    let budgets = vec![editable_budget(
        "SAVE", "", "Savings", "10%", "expense", "planned",
    )];

    validate_editable_budgets(&budgets).unwrap();
    let csv = serialize_editable_budgets(&budgets).unwrap();
    let parsed = parse_editable_budgets(&csv).unwrap();

    assert_eq!(parsed[0].monthly_budget, "10%");
    assert_eq!(parsed[0].yearly_budget, "");
    assert_eq!(parsed[0].income_basis, "planned");
}

#[test]
fn transfer_budget_code_forces_transfer_direction() {
    let budgets = vec![editable_budget(
        " transfer ",
        "",
        "Transfers",
        "0",
        "expense",
        "planned",
    )];

    let csv = serialize_editable_budgets(&budgets).unwrap();
    let parsed = parse_editable_budgets(&csv).unwrap();

    assert_eq!(parsed[0].code, "TRANSFER");
    assert_eq!(parsed[0].direction, "transfer");
    assert_eq!(parsed[0].income_basis, "real");
}

#[test]
fn refund_budget_codes_force_canonical_direction_and_basis() {
    let budgets = vec![
        editable_budget(" refunding ", "", "Refunding", "0", "income", "planned"),
        editable_budget(" refunded ", "", "Refunded", "0", "expense", "planned"),
    ];

    let csv = serialize_editable_budgets(&budgets).unwrap();
    let parsed = parse_editable_budgets(&csv).unwrap();

    assert_eq!(parsed[0].code, "REFUNDING");
    assert_eq!(parsed[0].direction, "expense");
    assert_eq!(parsed[0].income_basis, "real");
    assert_eq!(parsed[1].code, "REFUNDED");
    assert_eq!(parsed[1].direction, "income");
    assert_eq!(parsed[1].income_basis, "real");
}

#[test]
fn special_budget_aliases_keep_custom_codes_and_force_behavior() {
    let budgets = vec![
        editable_budget(
            "INTERNAL",
            "transfer",
            "Transfers",
            "0",
            "expense",
            "planned",
        ),
        editable_budget(
            "SALARY",
            "planned-income",
            "Income",
            "1000",
            "expense",
            "planned",
        ),
        editable_budget(
            "REFUND_OUT",
            "refunding",
            "Refunding",
            "0",
            "income",
            "planned",
        ),
        editable_budget(
            "REFUND_IN",
            "refunded",
            "Refunded",
            "0",
            "expense",
            "planned",
        ),
    ];

    let csv = serialize_editable_budgets(&budgets).unwrap();
    let parsed = parse_editable_budgets(&csv).unwrap();

    assert_eq!(parsed[0].code, "INTERNAL");
    assert_eq!(parsed[0].special, "transfer");
    assert_eq!(parsed[0].direction, "transfer");
    assert_eq!(parsed[0].income_basis, "real");
    assert_eq!(parsed[1].code, "SALARY");
    assert_eq!(parsed[1].special, "planned-income");
    assert_eq!(parsed[1].direction, "income");
    assert_eq!(parsed[2].code, "REFUND_OUT");
    assert_eq!(parsed[2].direction, "expense");
    assert_eq!(parsed[3].code, "REFUND_IN");
    assert_eq!(parsed[3].direction, "income");
}

#[test]
fn upsert_alias_adds_once() {
    let mut aliases = Vec::new();
    assert!(config::upsert_alias(&mut aliases, "date", "Boekdatum").unwrap());
    assert!(!config::upsert_alias(&mut aliases, "date", "boekdatum").unwrap());

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
        CsvCopyTarget::Transactions(1)
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

fn editable_budget(
    code: &str,
    special: &str,
    category: &str,
    monthly_budget: &str,
    direction: &str,
    income_basis: &str,
) -> EditableBudget {
    EditableBudget {
        code: code.to_string(),
        special: special.to_string(),
        category: category.to_string(),
        monthly_budget: monthly_budget.to_string(),
        yearly_budget: String::new(),
        direction: direction.to_string(),
        income_basis: income_basis.to_string(),
        notes: String::new(),
    }
}

fn write_test_configuration(dirs: &AppDirs, rules: &str, budgets: &str, aliases: &str) {
    ensure_layout(dirs).expect("create app dirs");
    fs::write(dirs.config.join("rules.csv"), rules).expect("write rules");
    fs::write(dirs.config.join("budgetcodes.csv"), budgets).expect("write budgets");
    fs::write(dirs.config.join("field_aliases.csv"), aliases).expect("write aliases");
}
