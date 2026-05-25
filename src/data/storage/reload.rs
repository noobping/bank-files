use super::super::*;
use super::period::{
    analytics_default_month, default_month_from_available_months, months_from_transactions,
    sort_transactions, years_from_months,
};
use crate::csv_detect::{import_files, FieldAliases};
use crate::model::{ImportOutcome, ImportReport};

pub fn reload_transaction_source_file(
    data: AppData,
    source: &TransactionSource,
    mode: DedupeMode,
    auto_clean_config: bool,
    remember_mode: RememberMode,
) -> Result<AppData> {
    let dirs = app_dirs()?;
    let capabilities = crate::data::storage_capabilities(&dirs);
    reload_transaction_source_file_with_dirs(
        data,
        &dirs,
        source,
        mode,
        auto_clean_config && capabilities.config_writable,
        remember_mode,
    )
}

#[cfg(test)]
pub(super) fn reload_inbox_file_with_dirs(
    data: AppData,
    dirs: &AppDirs,
    path: &Path,
    mode: DedupeMode,
    auto_clean_config: bool,
) -> Result<AppData> {
    let source = TransactionSource::inbox_file(path.to_path_buf());
    reload_transaction_source_file_with_dirs(
        data,
        dirs,
        &source,
        mode,
        auto_clean_config,
        RememberMode::DataOnly,
    )
}

fn reload_transaction_source_file_with_dirs(
    mut data: AppData,
    dirs: &AppDirs,
    source: &TransactionSource,
    mode: DedupeMode,
    auto_clean_config: bool,
    remember_mode: RememberMode,
) -> Result<AppData> {
    if auto_clean_config {
        remove_orphaned_rules()?;
    }

    let file = validated_transaction_source_file(source, dirs)?;
    let source_file = source_file_name(&file)?;
    let aliases = FieldAliases::load(&dirs.config)?;
    let outcome = import_files(std::slice::from_ref(&file), &aliases, data.loaded_scope)?;
    let rules = load_rules(&dirs.config)?;
    let budgets = load_budget_codes(&dirs.config)?;
    let ImportOutcome {
        transactions,
        reports,
        warnings,
        available_months,
        loaded_scope: _,
    } = outcome;

    data.transactions
        .retain(|transaction| transaction.source_file != source_file);
    data.transactions.extend(transactions);
    let (mut transactions, duplicate_count) = dedupe(std::mem::take(&mut data.transactions), mode);
    apply_rules(&mut transactions, &rules);
    sort_transactions(&mut transactions);
    data.transactions = transactions;

    data.reports
        .retain(|report| !report_matches_source(report, &file, &source_file));
    data.reports.extend(reports);
    data.reports
        .sort_by(|left, right| left.source.cmp(&right.source));

    data.warnings
        .retain(|warning| !warning_matches_source(warning, &file, &source_file));
    data.warnings.extend(warnings);

    let mut months = data
        .available_months
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    months.extend(available_months);
    if months.is_empty() {
        months.extend(months_from_transactions(&data.transactions));
    }
    data.available_months = months.into_iter().collect();
    data.available_years = years_from_months(&data.available_months);
    data.default_month = default_month_from_available_months(&data.available_months)
        .or_else(|| analytics_default_month(&data.transactions, &budgets));
    data.duplicate_count = duplicate_count;
    data.dedupe_mode = mode;
    data.budgets = budgets;
    data.rules_count = rules.iter().filter(|rule| rule.active).count();
    data.remember_mode = remember_mode;
    data.cache_status = DataCacheStatus::Disabled;
    if data.transaction_sources.is_empty() {
        data.transaction_sources = data
            .reports
            .iter()
            .map(|report| TransactionSource::inbox_file(report.source.clone()))
            .collect();
    }
    let reloaded_source = TransactionSource {
        kind: source.kind,
        path: file,
    };
    data.transaction_sources
        .retain(|existing| existing.path != reloaded_source.path);
    data.transaction_sources.push(reloaded_source);
    data.transaction_sources
        .sort_by(|left, right| left.path.cmp(&right.path));
    Ok(data)
}

fn validated_transaction_source_file(
    source: &TransactionSource,
    dirs: &AppDirs,
) -> Result<PathBuf> {
    match source.kind {
        TransactionSourceKind::InboxFile => validated_inbox_csv_file(source.path(), dirs),
        TransactionSourceKind::LiveFile => validated_live_csv_file(source.path()),
    }
}

fn validated_inbox_csv_file(path: &Path, dirs: &AppDirs) -> Result<PathBuf> {
    let inbox = dirs
        .inbox
        .canonicalize()
        .with_context(|| format!("Could not find app CSV folder: {}", dirs.inbox.display()))?;
    let file = path
        .canonicalize()
        .with_context(|| format!("Could not find CSV: {}", path.display()))?;

    if !file.starts_with(&inbox) {
        bail!(
            "This file is not in app storage and will not be reloaded: {}",
            path.display()
        );
    }
    if !is_csv(&file) {
        bail!("Only CSV files can be reloaded");
    }
    Ok(file)
}

fn validated_live_csv_file(path: &Path) -> Result<PathBuf> {
    let file = path
        .canonicalize()
        .with_context(|| format!("Could not find CSV: {}", path.display()))?;
    if !is_csv(&file) {
        bail!("Only CSV files can be reloaded");
    }
    Ok(file)
}

fn source_file_name(path: &Path) -> Result<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.trim().is_empty())
        .with_context(|| format!("Could not determine CSV file name: {}", path.display()))
}

fn report_matches_source(report: &ImportReport, file: &Path, source_file: &str) -> bool {
    report
        .source
        .canonicalize()
        .map(|source| source == file)
        .unwrap_or(false)
        || report
            .source
            .file_name()
            .map(|name| name.to_string_lossy().eq(source_file))
            .unwrap_or(false)
}

fn warning_matches_source(warning: &str, file: &Path, source_file: &str) -> bool {
    warning.starts_with(&format!("{}:", file.display())) || warning.starts_with(source_file)
}
