use super::*;
use crate::csv_detect::{import_files, FieldAliases};
use crate::model::{BudgetCode, ImportOutcome, ImportReport};
use crate::rules::Rule;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::io::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageCapabilities {
    pub data_readable: bool,
    pub data_writable: bool,
    pub config_readable: bool,
    pub config_writable: bool,
    pub data_reason: String,
    pub config_reason: String,
}

impl Default for StorageCapabilities {
    fn default() -> Self {
        Self {
            data_readable: true,
            data_writable: true,
            config_readable: true,
            config_writable: true,
            data_reason: String::new(),
            config_reason: String::new(),
        }
    }
}

impl StorageCapabilities {
    pub fn data_write_reason(&self) -> &str {
        if self.data_reason.is_empty() {
            "CSV storage is read-only."
        } else {
            &self.data_reason
        }
    }

    pub fn config_write_reason(&self) -> &str {
        if self.config_reason.is_empty() {
            "Configuration storage is read-only."
        } else {
            &self.config_reason
        }
    }
}

pub fn storage_capabilities(dirs: &AppDirs) -> StorageCapabilities {
    let (data_readable, data_writable, data_reason) =
        directory_capability(&dirs.inbox, "CSV storage");
    let (config_readable, config_writable, config_reason) =
        directory_capability(&dirs.config, "Configuration storage");
    StorageCapabilities {
        data_readable,
        data_writable,
        config_readable,
        config_writable,
        data_reason,
        config_reason,
    }
}

pub fn current_storage_capabilities() -> StorageCapabilities {
    app_dirs()
        .map(|dirs| crate::data::storage_capabilities(&dirs))
        .unwrap_or_else(|error| StorageCapabilities {
            data_readable: false,
            data_writable: false,
            config_readable: false,
            config_writable: false,
            data_reason: format!("Could not find app storage folders: {error:#}"),
            config_reason: format!("Could not find configuration folder: {error:#}"),
        })
}

fn directory_capability(path: &Path, label: &str) -> (bool, bool, String) {
    if path.exists() {
        let metadata = match fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                return (
                    false,
                    false,
                    format!("{label} cannot be inspected: {error}"),
                );
            }
        };
        if !metadata.is_dir() {
            return (false, false, format!("{label} is not a folder."));
        }
        let readable = fs::read_dir(path).is_ok();
        let writable = !metadata.permissions().readonly();
        let reason = if writable {
            String::new()
        } else {
            format!("{label} is read-only.")
        };
        return (readable, writable, reason);
    }

    match nearest_existing_parent(path) {
        Some(parent) => match fs::metadata(&parent) {
            Ok(metadata) if metadata.is_dir() && !metadata.permissions().readonly() => {
                (true, true, String::new())
            }
            Ok(_) => (
                true,
                false,
                format!("{label} cannot be created because its parent is read-only."),
            ),
            Err(error) => (true, false, format!("{label} cannot be inspected: {error}")),
        },
        None => (
            true,
            false,
            format!("{label} cannot be created because no parent folder exists."),
        ),
    }
}

fn nearest_existing_parent(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(parent) = current {
        if parent.exists() {
            return Some(parent.to_path_buf());
        }
        current = parent.parent();
    }
    None
}

pub fn mark_transaction_csv_readonly(path: &Path) -> Result<()> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Could not inspect imported CSV: {}", path.display()))?;
    let mut permissions = metadata.permissions();
    if !permissions.readonly() {
        permissions.set_readonly(true);
        fs::set_permissions(path, permissions).with_context(|| {
            format!("Could not mark imported CSV read-only: {}", path.display())
        })?;
    }
    Ok(())
}

fn mark_existing_transaction_csvs_readonly(dirs: &AppDirs) -> Vec<String> {
    let mut warnings = Vec::new();
    let Ok(entries) = fs::read_dir(&dirs.inbox) else {
        return warnings;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_csv(&path) {
            if let Err(error) = mark_transaction_csv_readonly(&path) {
                warnings.push(format!(
                    "Could not mark imported CSV read-only: {}: {error:#}",
                    path.display()
                ));
            }
        }
    }
    warnings
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CsvCopyResult {
    pub transaction_csvs: usize,
    pub config_csvs: usize,
    pub skipped: usize,
}

impl CsvCopyResult {
    pub fn imported(&self) -> usize {
        self.transaction_csvs + self.config_csvs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableRule {
    pub priority: i32,
    pub active: bool,
    pub field: String,
    pub search: String,
    pub is_regex: bool,
    pub category: String,
    pub budget_code: String,
    pub direction: String,
    pub amount_min: String,
    pub amount_max: String,
    pub notes: String,
}

impl EditableRule {
    pub fn new_default() -> Self {
        Self {
            priority: 120,
            active: true,
            field: "any".to_string(),
            search: String::new(),
            is_regex: false,
            category: "New category".to_string(),
            budget_code: "OTHER".to_string(),
            direction: "expense".to_string(),
            amount_min: String::new(),
            amount_max: String::new(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableBudget {
    pub code: String,
    pub category: String,
    pub monthly_budget: String,
    pub yearly_budget: String,
    pub direction: String,
    pub income_basis: String,
    pub notes: String,
}

impl EditableBudget {
    pub fn new_default() -> Self {
        Self {
            code: "NEW".to_string(),
            category: "New category".to_string(),
            monthly_budget: "0".to_string(),
            yearly_budget: String::new(),
            direction: "expense".to_string(),
            income_basis: "real".to_string(),
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EditableAlias {
    pub canonical: String,
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IgnoredTransactionPattern {
    pub key: String,
    pub label: String,
}

impl EditableAlias {
    pub fn new_default() -> Self {
        Self {
            canonical: "description".to_string(),
            alias: String::new(),
        }
    }
}

pub fn prepare_app_storage() -> Result<AppDirs> {
    let dirs = app_dirs()?;
    ensure_layout(&dirs)?;
    migrate_legacy_app_data_layout(&dirs)?;
    ensure_default_files(&dirs)?;
    Ok(dirs)
}

pub fn load_app_data_read_only_aware(
    mode: DedupeMode,
    auto_clean_config: bool,
    scope: TransactionLoadScope,
) -> Result<(AppData, StorageCapabilities)> {
    let dirs = app_dirs()?;
    let capabilities = crate::data::storage_capabilities(&dirs);
    load_app_data_from_dirs(
        &dirs,
        &capabilities,
        mode,
        auto_clean_config,
        scope,
        RememberMode::DataOnly,
        &[],
    )
}

pub fn load_app_data_with_sources(
    mode: DedupeMode,
    auto_clean_config: bool,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<(AppData, StorageCapabilities)> {
    let dirs = app_dirs()?;
    let capabilities = crate::data::storage_capabilities(&dirs);
    load_app_data_from_dirs(
        &dirs,
        &capabilities,
        mode,
        auto_clean_config,
        scope,
        remember_mode,
        sources,
    )
}

fn load_app_data_from_dirs(
    dirs: &AppDirs,
    capabilities: &StorageCapabilities,
    mode: DedupeMode,
    auto_clean_config: bool,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<(AppData, StorageCapabilities)> {
    let cache_key = remember_mode
        .uses_analytics_cache()
        .then(|| app_data_cache_key(dirs, mode, scope, remember_mode, sources))
        .transpose()?;

    if let Some(cache_key) = cache_key.as_deref() {
        match read_cached_app_data(cache_key) {
            Ok(Some(mut data)) => {
                data.cache_status = DataCacheStatus::Hit;
                return Ok((data, capabilities.clone()));
            }
            Ok(None) => {}
            Err(error) => {
                let mut data = load_app_data_uncached(
                    dirs,
                    capabilities,
                    mode,
                    auto_clean_config,
                    scope,
                    remember_mode,
                    sources,
                )?;
                data.warnings
                    .push(format!("Data and analytics cache ignored: {error:#}"));
                write_cache_status(cache_key, &mut data);
                return Ok((data, capabilities.clone()));
            }
        }
    }

    let mut data = load_app_data_uncached(
        dirs,
        capabilities,
        mode,
        auto_clean_config,
        scope,
        remember_mode,
        sources,
    )?;
    if let Some(cache_key) = cache_key.as_deref() {
        write_cache_status(cache_key, &mut data);
    }
    Ok((data, capabilities.clone()))
}

fn load_app_data_uncached(
    dirs: &AppDirs,
    capabilities: &StorageCapabilities,
    mode: DedupeMode,
    auto_clean_config: bool,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<AppData> {
    if auto_clean_config && capabilities.config_writable {
        remove_orphaned_rules()?;
    }
    let AppDataLoadInputs {
        mut outcome,
        rules,
        budgets,
        transaction_sources,
    } = load_app_data_inputs(dirs, scope, remember_mode, sources)?;
    if capabilities.data_writable && sources.is_empty() && !remember_mode.opens_live_files() {
        outcome
            .warnings
            .extend(mark_existing_transaction_csvs_readonly(dirs));
    }
    if auto_clean_config && !capabilities.config_writable {
        outcome.warnings.push(
            "Auto Clean Config skipped because configuration storage is read-only.".to_string(),
        );
    }
    let (mut transactions, duplicate_count) =
        dedupe(std::mem::take(&mut outcome.transactions), mode);
    apply_rules(&mut transactions, &rules, &budgets);
    sort_transactions(&mut transactions);

    let available_months = if outcome.available_months.is_empty() {
        months_from_transactions(&transactions)
    } else {
        outcome.available_months
    };
    let available_years = years_from_months(&available_months);
    let default_month = default_month_from_available_months(&available_months)
        .or_else(|| analytics_default_month(&transactions, &budgets));

    Ok(AppData {
        transactions,
        reports: outcome.reports,
        warnings: outcome.warnings,
        duplicate_count,
        dedupe_mode: mode,
        budgets,
        rules_count: rules.iter().filter(|r| r.active).count(),
        available_years,
        available_months,
        default_month,
        loaded_scope: outcome.loaded_scope,
        remember_mode,
        transaction_sources,
        cache_status: if remember_mode.uses_analytics_cache() {
            DataCacheStatus::Skipped
        } else {
            DataCacheStatus::Disabled
        },
    })
}

struct AppDataLoadInputs {
    outcome: ImportOutcome,
    rules: Vec<Rule>,
    budgets: Vec<BudgetCode>,
    transaction_sources: Vec<TransactionSource>,
}

fn load_app_data_inputs(
    dirs: &AppDirs,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<AppDataLoadInputs> {
    std::thread::scope(|thread_scope| {
        let import_handle =
            thread_scope.spawn(|| import_sources(dirs, scope, remember_mode, sources));
        let rules_handle = thread_scope.spawn(|| load_rules(&dirs.config));
        let budgets_handle = thread_scope.spawn(|| load_budget_codes(&dirs.config));

        let (outcome, transaction_sources) = import_handle
            .join()
            .map_err(|_| anyhow!("CSV import pipeline stopped unexpectedly"))??;
        let rules = rules_handle
            .join()
            .map_err(|_| anyhow!("Rules loading pipeline stopped unexpectedly"))??;
        let budgets = budgets_handle
            .join()
            .map_err(|_| anyhow!("Budget loading pipeline stopped unexpectedly"))??;

        Ok(AppDataLoadInputs {
            outcome,
            rules,
            budgets,
            transaction_sources,
        })
    })
}

fn import_sources(
    dirs: &AppDirs,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<(ImportOutcome, Vec<TransactionSource>)> {
    if sources.is_empty() && !remember_mode.opens_live_files() {
        let outcome = import_inbox(dirs, scope)?;
        let transaction_sources = outcome
            .reports
            .iter()
            .map(|report| TransactionSource::inbox_file(report.source.clone()))
            .collect();
        return Ok((outcome, transaction_sources));
    }

    let paths = sources
        .iter()
        .filter(|source| source.path().is_file() && is_csv(source.path()))
        .map(|source| source.path.clone())
        .collect::<Vec<_>>();
    let aliases = FieldAliases::load(&dirs.config)?;
    let outcome = import_files(&paths, &aliases, scope)?;
    let transaction_sources = sources
        .iter()
        .filter(|source| source.path().is_file() && is_csv(source.path()))
        .cloned()
        .collect();
    Ok((outcome, transaction_sources))
}

fn write_cache_status(cache_key: &str, data: &mut AppData) {
    match write_cached_app_data(cache_key, data) {
        Ok(()) => data.cache_status = DataCacheStatus::Updated,
        Err(error) => {
            data.cache_status = DataCacheStatus::Failed(format!("{error:#}"));
            data.warnings.push(format!(
                "Data and analytics cache was not updated: {error:#}"
            ));
        }
    }
}

fn read_cached_app_data(cache_key: &str) -> Result<Option<AppData>> {
    let path = app_data_cache_path(cache_key)?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "Could not read data and analytics cache: {}",
            path.display()
        )
    })?;
    let data = serde_json::from_str(&content).with_context(|| {
        format!(
            "Could not parse data and analytics cache: {}",
            path.display()
        )
    })?;
    Ok(Some(data))
}

pub fn clear_processed_app_data_cache() -> Result<bool> {
    let path = app_data_cache_root()?;
    match fs::remove_dir_all(&path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| {
            format!(
                "Could not remove data and analytics cache folder: {}",
                path.display()
            )
        }),
    }
}

fn write_cached_app_data(cache_key: &str, data: &AppData) -> Result<()> {
    let path = app_data_cache_path(cache_key)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Could not create data and analytics cache folder: {}",
                parent.display()
            )
        })?;
    }
    let temp = path.with_extension("json.tmp");
    let content =
        serde_json::to_string(data).context("Could not serialize data and analytics cache")?;
    fs::write(&temp, content).with_context(|| {
        format!(
            "Could not write data and analytics cache: {}",
            temp.display()
        )
    })?;
    fs::rename(&temp, &path).with_context(|| {
        format!(
            "Could not replace data and analytics cache: {}",
            path.display()
        )
    })?;
    Ok(())
}

fn app_data_cache_path(cache_key: &str) -> Result<PathBuf> {
    Ok(app_data_cache_root()?.join(format!("{cache_key}.json")))
}

fn app_data_cache_root() -> Result<PathBuf> {
    Ok(app_cache_dir()?.join("processed"))
}

fn app_data_cache_key(
    dirs: &AppDirs,
    mode: DedupeMode,
    scope: TransactionLoadScope,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<String> {
    let mut digest = Sha256::new();
    digest.update(env!("CARGO_PKG_VERSION").as_bytes());
    digest.update(format!("{mode:?}|{scope:?}|{remember_mode:?}").as_bytes());
    for source in effective_cache_sources(dirs, remember_mode, sources)? {
        digest.update(format!("{:?}|{}|", source.kind, source.path.display()).as_bytes());
        match fs::metadata(&source.path) {
            Ok(metadata) => {
                digest.update(metadata.len().to_string().as_bytes());
                if let Ok(modified) = metadata.modified() {
                    digest.update(format!("{modified:?}").as_bytes());
                }
            }
            Err(error) => digest.update(format!("missing:{error}").as_bytes()),
        }
    }
    for name in ["rules.csv", "budgetcodes.csv", "field_aliases.csv"] {
        let path = dirs.config.join(name);
        digest.update(path.display().to_string().as_bytes());
        match fs::metadata(&path) {
            Ok(metadata) => {
                digest.update(metadata.len().to_string().as_bytes());
                if let Ok(modified) = metadata.modified() {
                    digest.update(format!("{modified:?}").as_bytes());
                }
            }
            Err(_) => digest.update(b"missing"),
        }
    }
    Ok(hex::encode(digest.finalize()))
}

fn effective_cache_sources(
    dirs: &AppDirs,
    remember_mode: RememberMode,
    sources: &[TransactionSource],
) -> Result<Vec<TransactionSource>> {
    if !sources.is_empty() || remember_mode.opens_live_files() {
        let mut current = sources.to_vec();
        current.sort_by(|left, right| left.path.cmp(&right.path));
        return Ok(current);
    }

    let mut inbox_sources = inbox_csv_files(dirs)?
        .into_iter()
        .map(TransactionSource::inbox_file)
        .collect::<Vec<_>>();
    inbox_sources.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inbox_sources)
}

fn inbox_csv_files(dirs: &AppDirs) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if dirs.inbox.exists() {
        for entry in fs::read_dir(&dirs.inbox)
            .with_context(|| format!("Could not read CSV storage: {}", dirs.inbox.display()))?
        {
            let path = entry?.path();
            if path.is_file() && is_csv(&path) {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

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
fn reload_inbox_file_with_dirs(
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
    let outcome = import_files(&[file.clone()], &aliases, data.loaded_scope)?;
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
    apply_rules(&mut transactions, &rules, &budgets);
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

fn sort_transactions(transactions: &mut [Transaction]) {
    transactions.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| left.description.cmp(&right.description))
    });
}

fn months_from_transactions(transactions: &[Transaction]) -> Vec<MonthKey> {
    transactions
        .iter()
        .map(Transaction::month_key)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn years_from_months(months: &[MonthKey]) -> Vec<i32> {
    months
        .iter()
        .map(|month| month.year)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn default_month_from_available_months(months: &[MonthKey]) -> Option<MonthKey> {
    use chrono::Datelike;

    let latest = months.last().copied()?;
    let today = chrono::Local::now().date_naive();
    let current = MonthKey::new(today.year(), today.month());
    if months.contains(&current) {
        Some(current.previous())
    } else {
        Some(latest)
    }
}

fn analytics_default_month(
    transactions: &[Transaction],
    budgets: &[BudgetCode],
) -> Option<MonthKey> {
    crate::analytics::default_reporting_month(transactions, budgets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static CACHE_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn read_only_load_uses_defaults_without_creating_missing_config() {
        let root = unique_test_dir("read-only-load-defaults");
        fs::create_dir_all(&root).expect("test root should be created");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        let capabilities = StorageCapabilities {
            data_readable: true,
            data_writable: false,
            config_readable: true,
            config_writable: false,
            data_reason: "CSV storage is read-only.".to_string(),
            config_reason: "Configuration storage is read-only.".to_string(),
        };

        let (data, returned_capabilities) = load_app_data_from_dirs(
            &dirs,
            &capabilities,
            DedupeMode::Disabled,
            true,
            TransactionLoadScope::All,
            RememberMode::DataOnly,
            &[],
        )
        .expect("read-only load should use embedded defaults");

        assert!(data.transactions.is_empty());
        assert!(!dirs.config.exists());
        assert!(!dirs.inbox.exists());
        assert!(!data.budgets.is_empty());
        assert!(data
            .warnings
            .iter()
            .any(|warning| warning.contains("Auto Clean Config skipped")));
        assert_eq!(returned_capabilities, capabilities);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn forget_mode_loads_live_csv_without_creating_app_storage() {
        let root = unique_test_dir("forget-mode-live-csv");
        fs::create_dir_all(&root).expect("test root should be created");
        let live_csv = root.join("live.csv");
        fs::write(
            &live_csv,
            "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n",
        )
        .expect("live test csv should be written");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        let capabilities = StorageCapabilities {
            data_readable: true,
            data_writable: false,
            config_readable: true,
            config_writable: false,
            data_reason: "CSV storage is read-only.".to_string(),
            config_reason: "Configuration storage is read-only.".to_string(),
        };
        let sources = vec![TransactionSource::live_file(live_csv.clone())];

        let (data, returned_capabilities) = load_app_data_from_dirs(
            &dirs,
            &capabilities,
            DedupeMode::Disabled,
            false,
            TransactionLoadScope::All,
            RememberMode::Forget,
            &sources,
        )
        .expect("live load should read the selected CSV");

        assert_eq!(data.transactions.len(), 1);
        assert_eq!(data.remember_mode, RememberMode::Forget);
        assert_eq!(data.transaction_sources, sources);
        assert!(!dirs.config.exists());
        assert!(!dirs.inbox.exists());
        assert_eq!(returned_capabilities, capabilities);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn clear_processed_app_data_cache_removes_only_processed_cache() {
        let _guard = CACHE_ENV_LOCK
            .lock()
            .expect("cache env lock should be available");
        let root = unique_test_dir("clear-processed-cache");
        fs::create_dir_all(&root).expect("test root should be created");
        std::env::set_var("BANK_FILES_CACHE", root.join("cache"));
        let processed = root.join("cache").join("processed");
        let sibling = root.join("cache").join("other");
        fs::create_dir_all(&processed).expect("processed cache should be created");
        fs::create_dir_all(&sibling).expect("sibling cache should be created");
        fs::write(processed.join("entry.json"), "{}").expect("cache file should be written");
        fs::write(sibling.join("keep.txt"), "keep").expect("sibling file should be written");

        assert!(clear_processed_app_data_cache().expect("cache cleanup should succeed"));
        assert!(!processed.exists());
        assert!(sibling.exists());
        assert!(!clear_processed_app_data_cache().expect("missing cache is ok"));

        std::env::remove_var("BANK_FILES_CACHE");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn data_and_analytics_cache_reuses_processed_live_data() {
        let _guard = CACHE_ENV_LOCK
            .lock()
            .expect("cache env lock should be available");
        let root = unique_test_dir("full-cache-live-csv");
        fs::create_dir_all(&root).expect("test root should be created");
        std::env::set_var("BANK_FILES_CACHE", root.join("cache"));
        let live_csv = root.join("live.csv");
        fs::write(
            &live_csv,
            "Date,Description,Amount\n2026-01-01,Coffee,-2.50\n",
        )
        .expect("live test csv should be written");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        let capabilities = StorageCapabilities::default();
        let sources = vec![TransactionSource::live_file(live_csv)];

        let (first, _) = load_app_data_from_dirs(
            &dirs,
            &capabilities,
            DedupeMode::Disabled,
            false,
            TransactionLoadScope::All,
            RememberMode::DataAndAnalytics,
            &sources,
        )
        .expect("first full-cache load should parse and cache data");
        let (second, _) = load_app_data_from_dirs(
            &dirs,
            &capabilities,
            DedupeMode::Disabled,
            false,
            TransactionLoadScope::All,
            RememberMode::DataAndAnalytics,
            &sources,
        )
        .expect("second full-cache load should reuse cached data");

        assert_eq!(first.cache_status, DataCacheStatus::Updated);
        assert_eq!(second.cache_status, DataCacheStatus::Hit);
        assert_eq!(second.transactions.len(), 1);

        std::env::remove_var("BANK_FILES_CACHE");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn storage_capabilities_report_readonly_data_folder() {
        let root = unique_test_dir("readonly-data-capability");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        ensure_layout(&dirs).expect("test app dirs should be created");
        let mut permissions = fs::metadata(&dirs.data)
            .expect("data metadata should exist")
            .permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&dirs.data, permissions).expect("data folder should become read-only");

        let capabilities = crate::data::storage_capabilities(&dirs);

        assert!(!capabilities.data_writable);
        assert!(capabilities.config_writable);
        assert!(capabilities.data_write_reason().contains("read-only"));

        let mut permissions = fs::metadata(&dirs.data)
            .expect("data metadata should exist")
            .permissions();
        permissions.set_readonly(false);
        let _ = fs::set_permissions(&dirs.data, permissions);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn mark_transaction_csv_readonly_sets_readonly_permission() {
        let root = unique_test_dir("mark-csv-readonly");
        fs::create_dir_all(&root).expect("test root should be created");
        let csv = root.join("transactions.csv");
        fs::write(
            &csv,
            "Date,Description,Amount
2026-01-01,Coffee,-2.50
",
        )
        .expect("test csv should be written");

        mark_transaction_csv_readonly(&csv).expect("csv should be marked read-only");

        assert!(fs::metadata(&csv)
            .expect("csv metadata should exist")
            .permissions()
            .readonly());

        let mut permissions = fs::metadata(&csv)
            .expect("csv metadata should exist")
            .permissions();
        permissions.set_readonly(false);
        let _ = fs::set_permissions(&csv, permissions);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn reload_inbox_file_replaces_only_selected_file() {
        let root = unique_test_dir("reload-inbox-file");
        let dirs = AppDirs {
            config: root.join("config"),
            data: root.join("data"),
            inbox: root.join("data"),
        };
        ensure_layout(&dirs).expect("test app dirs should be created");
        ensure_default_files(&dirs).expect("test config should be created");

        let selected_csv = dirs.inbox.join("selected.csv");
        let other_csv = dirs.inbox.join("other.csv");
        fs::write(
            &selected_csv,
            "Date,Description,Amount\n2026-01-03,Fresh selected,-12.34\n",
        )
        .expect("selected test csv should be written");
        fs::write(
            &other_csv,
            "Date,Description,Amount\n2026-01-04,Fresh other,-8.00\n",
        )
        .expect("other test csv should be written");
        mark_transaction_csv_readonly(&selected_csv).expect("selected csv should be read-only");

        let old_warning = format!("{}: old selected warning", selected_csv.display());
        let other_warning = format!("{}: keep other warning", other_csv.display());
        let data = AppData {
            transactions: vec![
                transaction("selected.csv", "Old selected"),
                transaction("other.csv", "Keep other"),
            ],
            reports: vec![import_report(&selected_csv), import_report(&other_csv)],
            warnings: vec![old_warning.clone(), other_warning.clone()],
            dedupe_mode: DedupeMode::Disabled,
            available_months: vec![MonthKey::new(2026, 1)],
            available_years: vec![2026],
            default_month: Some(MonthKey::new(2026, 1)),
            loaded_scope: TransactionLoadScope::All,
            ..AppData::default()
        };

        let reloaded =
            reload_inbox_file_with_dirs(data, &dirs, &selected_csv, DedupeMode::Disabled, false)
                .expect("selected csv should reload");

        assert!(reloaded
            .transactions
            .iter()
            .any(|transaction| transaction.source_file == "selected.csv"
                && transaction.description == "Fresh selected"));
        assert!(!reloaded
            .transactions
            .iter()
            .any(|transaction| transaction.source_file == "selected.csv"
                && transaction.description == "Old selected"));
        assert!(reloaded
            .transactions
            .iter()
            .any(|transaction| transaction.source_file == "other.csv"
                && transaction.description == "Keep other"));
        assert!(reloaded
            .reports
            .iter()
            .any(|report| report.source == selected_csv && report.rows_imported == 1));
        assert!(!reloaded.warnings.contains(&old_warning));
        assert!(reloaded.warnings.contains(&other_warning));

        fs::remove_dir_all(root).ok();
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("bank-files-{name}-{suffix}"))
    }

    fn import_report(source: &Path) -> ImportReport {
        ImportReport {
            source: source.to_path_buf(),
            rows_seen: 1,
            rows_imported: 1,
            ..ImportReport::default()
        }
    }

    fn transaction(source_file: &str, description: &str) -> Transaction {
        Transaction {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid test date"),
            amount: Decimal::new(-1000, 2),
            description: description.to_string(),
            counterparty: String::new(),
            tags: String::new(),
            account: String::new(),
            transaction_id: String::new(),
            currency: "EUR".to_string(),
            source_file: source_file.to_string(),
            source_row: 2,
            category: "Uncategorized".to_string(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: format!("strict-{source_file}-{description}"),
            loose_key: format!("loose-{source_file}-{description}"),
        }
    }
}
