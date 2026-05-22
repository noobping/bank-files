use crate::csv_detect::import_inbox;
use crate::model::{
    AppData, BudgetAmount, DedupeMode, MonthKey, Transaction, TransactionLoadScope,
};
use crate::rules::{apply_rules, load_budget_codes, load_rules};
use crate::util::{app_dirs, ensure_layout, normalize_key, parse_decimal, AppDirs};
use adw::gtk::gio::prelude::FileExt;
use anyhow::{bail, Context, Result};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

mod config;
mod copy;
mod defaults;
mod editable;
mod export;
mod generate;
mod maintenance;
mod storage;
#[cfg(test)]
mod tests;
mod validation;

pub use config::{
    archive_configuration, combine_editable_rules, configuration_archive_exists,
    group_editable_rules_for_combining, ignore_transaction_pattern,
    ignored_transaction_pattern_keys, load_editable_aliases, load_editable_budgets,
    load_editable_rules, orphaned_rules, remove_orphaned_rules, reopen_transaction_pattern,
    restore_configuration_archive, restore_default_configuration, restore_empty_configuration,
    upsert_editable_alias, write_editable_aliases, write_editable_budgets, write_editable_rules,
    write_generated_configuration, OrphanedRule,
};
pub use copy::{copy_files_to_app_storage, copy_uris_to_app_storage};
pub use export::{export_transactions_to_path, remove_inbox_file};
pub use generate::{
    generate_automatic_configuration, generated_budget_code_for_category, GeneratedConfiguration,
    GeneratedConfigurationSummary,
};
pub use storage::{
    current_storage_capabilities, load_app_data_read_only_aware, load_app_data_with_config_cleanup,
    mark_transaction_csv_readonly, prepare_app_storage, reload_inbox_file, storage_capabilities,
    CsvCopyResult, EditableAlias, EditableBudget, EditableRule, IgnoredTransactionPattern,
    StorageCapabilities,
};

use copy::unique_inbox_target;
use defaults::{default_aliases, default_budgets, default_rules, FALSE_ALIASES};
use editable::{
    parse_editable_aliases, parse_editable_budgets, parse_editable_rules,
    serialize_editable_aliases, serialize_editable_budgets, serialize_editable_rules,
};
use maintenance::{dedupe, ensure_default_files, is_csv, migrate_legacy_app_data_layout};
use validation::{
    csv_cell, form_search_from_pattern, non_empty, parse_bool_cell, pattern_from_form,
    writer_to_string,
};
use validation::{validate_editable_budgets, validate_editable_rules};

pub(crate) fn editable_rules_to_csv(rules: &[EditableRule]) -> Result<String> {
    serialize_editable_rules(rules)
}

pub(crate) fn editable_budgets_to_csv(budgets: &[EditableBudget]) -> Result<String> {
    serialize_editable_budgets(budgets)
}

pub(crate) fn editable_aliases_to_csv(aliases: &[EditableAlias]) -> Result<String> {
    serialize_editable_aliases(aliases)
}

#[cfg(test)]
use copy::{config_csv_from_headers, config_csv_name, copy_gio_file_to_app_storage, CsvCopyTarget};
