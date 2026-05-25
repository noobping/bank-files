use super::super::*;

pub fn read_config_file(name: &str) -> Result<(PathBuf, String)> {
    let dirs = app_dirs()?;
    let path = config_file_path(&dirs, name)?;
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            default_config_contents(name)?.to_string()
        }
        Err(error) => {
            return Err(error).with_context(|| format!("Could not read {}", path.display()))
        }
    };
    Ok((path, contents))
}

fn default_config_contents(name: &str) -> Result<&'static str> {
    match name {
        "rules.csv" => Ok(default_rules()),
        "budgetcodes.csv" => Ok(default_budgets()),
        "field_aliases.csv" => Ok(default_aliases()),
        _ => anyhow::bail!("Unknown configuration file: {name}"),
    }
}

pub fn write_config_file(name: &str, contents: &str) -> Result<PathBuf> {
    let dirs = prepare_app_storage()?;
    let path = config_file_path(&dirs, name)?;
    fs::write(&path, contents).with_context(|| format!("Could not save {}", path.display()))?;
    Ok(path)
}

pub(in crate::data) fn config_file_path(dirs: &AppDirs, name: &str) -> Result<PathBuf> {
    match name {
        "rules.csv" | "budgetcodes.csv" | "field_aliases.csv" => Ok(dirs.config.join(name)),
        _ => anyhow::bail!("Unknown configuration file: {name}"),
    }
}

pub(super) struct ConfigurationContents<'a> {
    pub(super) rules: &'a str,
    pub(super) budgets: &'a str,
    pub(super) aliases: &'a str,
}

pub(in crate::data) fn restore_default_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    write_configuration_contents(
        dirs,
        ConfigurationContents {
            rules: default_rules(),
            budgets: default_budgets(),
            aliases: default_aliases(),
        },
    )
}

pub(in crate::data) fn restore_empty_configuration_in(dirs: &AppDirs) -> Result<PathBuf> {
    let rules = serialize_editable_rules(&[])?;
    let budgets = serialize_editable_budgets(&[])?;
    write_configuration_contents(
        dirs,
        ConfigurationContents {
            rules: &rules,
            budgets: &budgets,
            aliases: default_aliases(),
        },
    )
}

pub(super) fn write_configuration_contents(
    dirs: &AppDirs,
    contents: ConfigurationContents<'_>,
) -> Result<PathBuf> {
    ensure_layout(dirs)?;

    fs::write(config_file_path(dirs, "rules.csv")?, contents.rules)
        .with_context(|| "Could not write rules configuration".to_string())?;
    fs::write(config_file_path(dirs, "budgetcodes.csv")?, contents.budgets)
        .with_context(|| "Could not write budget configuration".to_string())?;
    fs::write(
        config_file_path(dirs, "field_aliases.csv")?,
        contents.aliases,
    )
    .with_context(|| "Could not write field name configuration".to_string())?;

    Ok(dirs.config.clone())
}
