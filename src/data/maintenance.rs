use super::*;

pub(in crate::data) fn dedupe(
    transactions: Vec<Transaction>,
    mode: DedupeMode,
) -> (Vec<Transaction>, usize) {
    if !mode.is_enabled() {
        return (transactions, 0);
    }

    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(transactions.len());
    let mut duplicates = 0usize;

    for tx in transactions {
        let key = tx.strict_key.clone();
        if seen.insert(key) {
            out.push(tx);
        } else {
            duplicates += 1;
        }
    }
    (out, duplicates)
}

pub(in crate::data) fn ensure_default_files(dirs: &AppDirs) -> Result<()> {
    write_if_missing(&dirs.config.join("rules.csv"), default_rules())?;
    write_if_missing(&dirs.config.join("budgetcodes.csv"), default_budgets())?;
    write_if_missing(&dirs.config.join("field_aliases.csv"), default_aliases())?;
    write_if_missing(
        &dirs.config.join("ignored_transaction_patterns.csv"),
        "key,label\n",
    )?;
    Ok(())
}

pub(in crate::data) fn is_csv(path: &Path) -> bool {
    path.extension()
        .map(|ext| ext.eq_ignore_ascii_case("csv"))
        .unwrap_or(false)
}

pub(in crate::data) fn write_if_missing(path: &Path, contents: &str) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }
    Ok(())
}

pub(in crate::data) fn migrate_legacy_app_data_layout(dirs: &AppDirs) -> Result<()> {
    let legacy_inbox = dirs.data.join("inbox");
    if legacy_inbox.is_dir() {
        for entry in fs::read_dir(&legacy_inbox)
            .with_context(|| format!("Could not read legacy inbox: {}", legacy_inbox.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || !is_csv(&path) {
                continue;
            }

            let Some(name) = path.file_name().map(PathBuf::from) else {
                continue;
            };
            let target = unique_inbox_target(&dirs.inbox, &name);
            fs::rename(&path, &target).with_context(|| {
                format!(
                    "Could not move legacy CSV from {} to {}",
                    path.display(),
                    target.display()
                )
            })?;
        }
        let _ = fs::remove_dir(&legacy_inbox);
    }

    let legacy_exports = dirs.data.join("exports");
    let _ = fs::remove_dir(&legacy_exports);

    Ok(())
}
