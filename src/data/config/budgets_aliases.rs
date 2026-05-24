use super::super::*;
use super::files::{
    archive_configuration_in, configuration_archive_exists_in, read_config_file,
    restore_configuration_archive_in, restore_default_configuration_in,
    restore_empty_configuration_in, write_config_file, write_configuration_contents,
    ConfigurationContents,
};
use super::ignored_patterns::serialize_ignored_transaction_patterns;

pub fn load_editable_budgets() -> Result<Vec<EditableBudget>> {
    let (_, contents) = read_config_file("budgetcodes.csv")?;
    parse_editable_budgets(&contents)
}

pub fn write_editable_budgets(budgets: &[EditableBudget]) -> Result<PathBuf> {
    validate_editable_budgets(budgets)?;
    let contents = serialize_editable_budgets(budgets)?;
    write_config_file("budgetcodes.csv", &contents)
}

pub fn restore_default_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_default_configuration_in(&dirs)
}

pub fn restore_empty_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_empty_configuration_in(&dirs)
}

pub fn configuration_archive_exists() -> Result<bool> {
    let dirs = app_dirs()?;
    Ok(configuration_archive_exists_in(&dirs))
}

pub fn archive_configuration() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    archive_configuration_in(&dirs)
}

pub fn restore_configuration_archive() -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    restore_configuration_archive_in(&dirs)
}

pub fn load_editable_aliases() -> Result<Vec<EditableAlias>> {
    let (_, contents) = read_config_file("field_aliases.csv")?;
    parse_editable_aliases(&contents)
}

pub fn write_editable_aliases(aliases: &[EditableAlias]) -> Result<PathBuf> {
    let contents = serialize_editable_aliases(aliases)?;
    write_config_file("field_aliases.csv", &contents)
}

pub fn write_generated_configuration(config: &GeneratedConfiguration) -> Result<PathBuf> {
    let rules = serialize_editable_rules(&config.rules)?;
    let budgets = serialize_editable_budgets(&config.budgets)?;
    let aliases = serialize_editable_aliases(&config.aliases)?;
    let ignored_patterns = serialize_ignored_transaction_patterns(&config.ignored_patterns)?;
    let dirs = prepare_app_storage()?;
    write_configuration_contents(
        &dirs,
        ConfigurationContents {
            rules: &rules,
            budgets: &budgets,
            aliases: &aliases,
            ignored_patterns: &ignored_patterns,
        },
    )
}

pub fn upsert_editable_alias(canonical: &str, alias: &str) -> Result<bool> {
    let mut aliases = load_editable_aliases()?;
    if !upsert_alias(&mut aliases, canonical, alias)? {
        return Ok(false);
    }

    write_editable_aliases(&aliases)?;
    Ok(true)
}

pub(in crate::data) fn upsert_alias(
    aliases: &mut Vec<EditableAlias>,
    canonical: &str,
    alias: &str,
) -> Result<bool> {
    let canonical = canonical.trim();
    let alias = alias.trim();
    if canonical.is_empty() || alias.is_empty() {
        anyhow::bail!("Field alias needs both an app field and a CSV header");
    }

    let already_exists = aliases.iter().any(|existing| {
        existing.canonical.trim() == canonical && existing.alias.trim().eq_ignore_ascii_case(alias)
    });
    if already_exists {
        return Ok(false);
    }

    aliases.push(EditableAlias {
        canonical: canonical.to_string(),
        alias: alias.to_string(),
    });
    Ok(true)
}
