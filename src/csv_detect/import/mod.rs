use super::*;

mod csv_io;
mod parallel;
mod plan;
mod scope;

use parallel::run_indexed_in_parallel;
use plan::{import_planned_csv, plan_csv};
use scope::default_month_from_available_months;

#[cfg(test)]
pub(super) use parallel::parallel_chunk_ranges;

pub fn import_inbox(dirs: &AppDirs, scope: TransactionLoadScope) -> Result<ImportOutcome> {
    let aliases = FieldAliases::load(&dirs.config)?;
    let mut files: Vec<PathBuf> = Vec::new();
    if dirs.inbox.exists() {
        for entry in WalkDir::new(&dirs.inbox).max_depth(1).into_iter().flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map(|e| e.eq_ignore_ascii_case("csv"))
                    .unwrap_or(false)
            {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    import_files(&files, &aliases, scope)
}

pub fn import_files(
    files: &[PathBuf],
    aliases: &FieldAliases,
    requested_scope: TransactionLoadScope,
) -> Result<ImportOutcome> {
    let mut outcome = ImportOutcome::default();
    let mut plans = Vec::new();
    let mut available_months = BTreeSet::new();

    for (index, result) in run_indexed_in_parallel(
        files.len(),
        |index| plan_csv(&files[index], aliases),
        "CSV planning worker stopped unexpectedly",
    )? {
        match result {
            Ok(plan) => {
                available_months.extend(plan.available_months.iter().copied());
                plans.push(plan);
            }
            Err(err) => outcome
                .warnings
                .push(format!("{}: {err:#}", files[index].display())),
        }
    }

    outcome.available_months = available_months.into_iter().collect();
    let default_month = default_month_from_available_months(&outcome.available_months);
    let resolved_scope = requested_scope.resolve(default_month);
    outcome.loaded_scope = resolved_scope;

    for (index, result) in run_indexed_in_parallel(
        plans.len(),
        |index| import_planned_csv(&plans[index], resolved_scope),
        "CSV import worker stopped unexpectedly",
    )? {
        match result {
            Ok((transactions, report)) => {
                outcome.transactions.extend(transactions);
                outcome.reports.push(report);
            }
            Err(err) => outcome
                .warnings
                .push(format!("{}: {err:#}", plans[index].path.display())),
        }
    }

    Ok(outcome)
}
