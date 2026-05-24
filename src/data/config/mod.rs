mod budgets_aliases;
mod files;
mod ignored_patterns;
mod rule_search;
mod rule_terms;
mod rules;

#[cfg(test)]
pub(in crate::data) use budgets_aliases::upsert_alias;
pub use budgets_aliases::{
    archive_configuration, configuration_archive_exists, load_editable_aliases,
    load_editable_budgets, restore_configuration_archive, restore_default_configuration,
    restore_empty_configuration, upsert_editable_alias, write_editable_aliases,
    write_editable_budgets, write_generated_configuration,
};
#[cfg(test)]
pub(in crate::data) use files::{
    archive_configuration_in, restore_configuration_archive_in, restore_default_configuration_in,
    restore_empty_configuration_in,
};
pub use ignored_patterns::{
    ignore_transaction_pattern, ignored_transaction_pattern_keys, reopen_transaction_pattern,
};
pub use rules::{
    combine_editable_rules, group_editable_rules_for_combining, load_editable_rules,
    orphaned_rules, remove_orphaned_rules, write_editable_rules, OrphanedRule,
};

pub(super) const CONFIG_FILE_NAMES: [&str; 4] = [
    "rules.csv",
    "budgetcodes.csv",
    "field_aliases.csv",
    "ignored_transaction_patterns.csv",
];
pub(super) const CONFIG_ARCHIVE_DIR: &str = "archive";
pub(super) const EMPTY_IGNORED_TRANSACTION_PATTERNS: &str = "key,label\n";
