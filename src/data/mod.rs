use crate::csv_detect::import_inbox;
use crate::model::{
    AppData, BudgetAmount, DataCacheStatus, DedupeMode, MonthKey, RememberMode, Transaction,
    TransactionLoadScope, TransactionSource, TransactionSourceKind,
};
use crate::rules::{apply_rules, load_budget_codes, load_rules};
use crate::util::{app_cache_dir, app_dirs, ensure_layout, normalize_key, parse_decimal, AppDirs};
use adw::gtk::gio::prelude::FileExt;
use anyhow::{bail, Context, Result};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

mod budget_code;
mod config;
mod copy;
mod defaults;
mod editable;
mod export;
mod maintenance;
mod storage;
#[cfg(test)]
mod tests;
mod validation;

pub use budget_code::generated_budget_code_for_category;
pub use config::{
    archive_configuration, combine_editable_rules, configuration_archive_exists,
    configuration_archives, editable_rule_literal_terms, group_editable_rules_for_combining,
    load_editable_aliases, load_editable_budgets, load_editable_rules, orphaned_rules,
    remove_configuration_archive, remove_orphaned_rules, restore_configuration_archive,
    restore_configuration_archive_by_id, restore_default_configuration,
    restore_empty_configuration, rule_search_from_literal_terms, upsert_editable_alias,
    write_editable_aliases, write_editable_budgets, write_editable_rules, ConfigurationArchive,
    OrphanedRule,
};
pub use copy::{copy_files_to_app_storage, copy_uris_to_app_storage};
pub use export::{export_transactions_to_path, remove_inbox_file};
pub use storage::{
    clear_processed_app_data_cache, current_storage_capabilities, load_app_data_read_only_aware,
    load_app_data_with_sources, mark_transaction_csv_readonly, prepare_app_storage,
    reload_transaction_source_file, storage_capabilities, CsvCopyResult, EditableAlias,
    EditableBudget, EditableRule, StorageCapabilities,
};

use defaults::{default_aliases, default_budgets, default_rules, FALSE_ALIASES};
use editable::{
    parse_editable_aliases, parse_editable_budgets, parse_editable_rules,
    serialize_editable_aliases, serialize_editable_budgets, serialize_editable_rules,
};
use maintenance::{dedupe, ensure_default_files, is_csv};
use validation::{csv_cell, non_empty, parse_bool_cell, writer_to_string};
pub(crate) use validation::{form_search_from_pattern, pattern_from_form};
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
