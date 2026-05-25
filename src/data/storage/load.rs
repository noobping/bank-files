use super::super::*;
use super::cache::{app_data_cache_key, read_cached_app_data, write_cache_status};
use super::capabilities::{mark_existing_transaction_csvs_readonly, StorageCapabilities};
use super::period::{
    analytics_default_month, default_month_from_available_months, months_from_transactions,
    sort_transactions, years_from_months,
};
use crate::csv_detect::{import_files, FieldAliases};
use crate::model::{BudgetCode, ImportOutcome};
use crate::rules::Rule;
use anyhow::anyhow;

pub fn prepare_app_storage() -> Result<AppDirs> {
    let dirs = app_dirs()?;
    ensure_layout(&dirs)?;
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
        AppDataLoadRequest {
            mode,
            auto_clean_config,
            scope,
            remember_mode: RememberMode::DataOnly,
            sources: &[],
        },
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
        AppDataLoadRequest {
            mode,
            auto_clean_config,
            scope,
            remember_mode,
            sources,
        },
    )
}

#[derive(Clone, Copy)]
pub(super) struct AppDataLoadRequest<'a> {
    pub(super) mode: DedupeMode,
    pub(super) auto_clean_config: bool,
    pub(super) scope: TransactionLoadScope,
    pub(super) remember_mode: RememberMode,
    pub(super) sources: &'a [TransactionSource],
}

pub(super) fn load_app_data_from_dirs(
    dirs: &AppDirs,
    capabilities: &StorageCapabilities,
    request: AppDataLoadRequest<'_>,
) -> Result<(AppData, StorageCapabilities)> {
    let cache_key = request
        .remember_mode
        .uses_analytics_cache()
        .then(|| {
            app_data_cache_key(
                dirs,
                request.mode,
                request.scope,
                request.remember_mode,
                request.sources,
            )
        })
        .transpose()?;

    if let Some(cache_key) = cache_key.as_deref() {
        match read_cached_app_data(cache_key) {
            Ok(Some(mut data)) => {
                data.cache_status = DataCacheStatus::Hit;
                return Ok((data, capabilities.clone()));
            }
            Ok(None) => {}
            Err(error) => {
                let mut data = load_app_data_uncached(dirs, capabilities, request)?;
                data.warnings
                    .push(format!("Data and analytics cache ignored: {error:#}"));
                write_cache_status(cache_key, &mut data);
                return Ok((data, capabilities.clone()));
            }
        }
    }

    let mut data = load_app_data_uncached(dirs, capabilities, request)?;
    if let Some(cache_key) = cache_key.as_deref() {
        write_cache_status(cache_key, &mut data);
    }
    Ok((data, capabilities.clone()))
}

fn load_app_data_uncached(
    dirs: &AppDirs,
    capabilities: &StorageCapabilities,
    request: AppDataLoadRequest<'_>,
) -> Result<AppData> {
    if request.auto_clean_config && capabilities.config_writable {
        remove_orphaned_rules()?;
    }
    let AppDataLoadInputs {
        mut outcome,
        rules,
        budgets,
        transaction_sources,
    } = load_app_data_inputs(dirs, request.scope, request.remember_mode, request.sources)?;
    if capabilities.data_writable
        && request.sources.is_empty()
        && !request.remember_mode.opens_live_files()
    {
        outcome
            .warnings
            .extend(mark_existing_transaction_csvs_readonly(dirs));
    }
    if request.auto_clean_config && !capabilities.config_writable {
        outcome.warnings.push(
            "Auto Clean Config skipped because configuration storage is read-only.".to_string(),
        );
    }
    let (mut transactions, duplicate_count) =
        dedupe(std::mem::take(&mut outcome.transactions), request.mode);
    apply_rules(&mut transactions, &rules);
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
        dedupe_mode: request.mode,
        budgets,
        rules_count: rules.iter().filter(|r| r.active).count(),
        available_years,
        available_months,
        default_month,
        loaded_scope: outcome.loaded_scope,
        remember_mode: request.remember_mode,
        transaction_sources,
        cache_status: if request.remember_mode.uses_analytics_cache() {
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
