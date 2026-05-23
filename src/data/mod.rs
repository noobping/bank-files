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
    clear_processed_app_data_cache, current_storage_capabilities, load_app_data_read_only_aware,
    load_app_data_with_sources, mark_transaction_csv_readonly, prepare_app_storage,
    reload_transaction_source_file, storage_capabilities, CsvCopyResult, EditableAlias,
    EditableBudget, EditableRule, IgnoredTransactionPattern, StorageCapabilities,
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

pub(crate) fn validate_generated_configuration(config: &GeneratedConfiguration) -> Result<()> {
    validate_editable_budgets(&config.budgets)?;
    validate_generated_budgets(&config.budgets)?;
    validate_editable_rules(&config.rules)?;
    validate_generated_rules(&config.rules)?;
    validate_editable_aliases(&config.aliases)?;
    validate_ignored_transaction_patterns(&config.ignored_patterns)?;
    let _ = serialize_editable_budgets(&config.budgets)?;
    let _ = serialize_editable_rules(&config.rules)?;
    let _ = serialize_editable_aliases(&config.aliases)?;
    Ok(())
}

fn validate_generated_budgets(budgets: &[EditableBudget]) -> Result<()> {
    let mut codes = BTreeSet::<String>::new();
    for (index, budget) in budgets.iter().enumerate() {
        let code = budget.code.trim();
        if code.is_empty() || budget.category.trim().is_empty() {
            anyhow::bail!("Budget {} needs both a code and category", index + 1);
        }
        if !codes.insert(normalize_key(code)) {
            anyhow::bail!("Budget {} duplicates an earlier budget code", index + 1);
        }
    }
    Ok(())
}

fn validate_generated_rules(rules: &[EditableRule]) -> Result<()> {
    for (index, rule) in rules.iter().enumerate() {
        if rule.search.trim().is_empty()
            || rule.category.trim().is_empty()
            || rule.budget_code.trim().is_empty()
        {
            anyhow::bail!(
                "Rule {} needs search text, category, and budget code",
                index + 1
            );
        }
        if !matches!(
            rule.field.trim(),
            "any" | "tags" | "description" | "counterparty" | "account" | "transaction_id"
        ) {
            anyhow::bail!("Rule {} has an unknown field", index + 1);
        }
    }
    Ok(())
}

fn validate_editable_aliases(aliases: &[EditableAlias]) -> Result<()> {
    let mut keys = BTreeSet::<(String, String)>::new();
    for (index, alias) in aliases.iter().enumerate() {
        let canonical = alias.canonical.trim();
        let csv_alias = alias.alias.trim();
        if canonical.is_empty() || csv_alias.is_empty() {
            anyhow::bail!(
                "Alias {} needs both an app field and a CSV header",
                index + 1
            );
        }
        if !keys.insert((normalize_key(canonical), normalize_key(csv_alias))) {
            anyhow::bail!("Alias {} duplicates an earlier field mapping", index + 1);
        }
    }
    Ok(())
}

fn validate_ignored_transaction_patterns(patterns: &[IgnoredTransactionPattern]) -> Result<()> {
    let mut keys = BTreeSet::<String>::new();
    for (index, pattern) in patterns.iter().enumerate() {
        let key = pattern.key.trim();
        if key.is_empty() || pattern.label.trim().is_empty() {
            anyhow::bail!("Ignored pattern {} needs both a key and label", index + 1);
        }
        if !keys.insert(key.to_string()) {
            anyhow::bail!(
                "Ignored pattern {} duplicates an earlier pattern",
                index + 1
            );
        }
    }
    Ok(())
}

#[cfg(test)]
use copy::{config_csv_from_headers, config_csv_name, copy_gio_file_to_app_storage, CsvCopyTarget};
