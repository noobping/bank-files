mod backup;
mod budgets_aliases;
mod files;
mod rule_search;
mod rule_terms;
mod rules;

pub use backup::ConfigurationArchive;
#[cfg(test)]
pub(in crate::data) use backup::{
    archive_configuration_in, configuration_archives_in, remove_configuration_archive_in,
    restore_configuration_archive_by_id_in, restore_configuration_archive_in,
};
#[cfg(test)]
pub(in crate::data) use budgets_aliases::upsert_alias;
pub use budgets_aliases::{
    archive_configuration, configuration_archive_exists, configuration_archives,
    load_editable_aliases, load_editable_budgets, remove_configuration_archive,
    restore_configuration_archive, restore_configuration_archive_by_id,
    restore_default_configuration, restore_empty_configuration, upsert_editable_alias,
    write_editable_aliases, write_editable_budgets,
};
#[cfg(test)]
pub(in crate::data) use files::{restore_default_configuration_in, restore_empty_configuration_in};
pub use rules::{
    combine_editable_rules, editable_rule_literal_terms, group_editable_rules_for_combining,
    load_editable_rules, orphaned_rules, remove_orphaned_rules, rule_search_from_literal_terms,
    write_editable_rules, OrphanedRule,
};

pub(super) const CONFIG_FILE_NAMES: [&str; 3] =
    ["rules.csv", "budgetcodes.csv", "field_aliases.csv"];
pub(super) const CONFIG_ARCHIVE_DIR: &str = "archive";
