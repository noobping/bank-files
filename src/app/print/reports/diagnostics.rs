use super::*;

pub(super) fn diagnostics_print_report(
    data: &AppData,
    search: Option<&SearchFilter>,
) -> PrintReport {
    let visible_reports: Vec<_> = data
        .reports
        .iter()
        .filter(|report| diagnostic_report_matches(report, search))
        .collect();
    let rows_seen: usize = visible_reports.iter().map(|report| report.rows_seen).sum();
    let rows_imported: usize = visible_reports
        .iter()
        .map(|report| report.rows_imported)
        .sum();
    let rows_skipped: usize = visible_reports
        .iter()
        .map(|report| report.rows_skipped)
        .sum();
    let unconfigured_budget_count =
        analytics::unconfigured_expense_budget_count(&data.transactions, &data.budgets);
    let other_category_count = analytics::other_category_count(&data.transactions);
    let warnings = filtered_warnings(&data.warnings, search);
    let mut sections = vec![diagnostic_files_print_table(&visible_reports)];
    if !warnings.is_empty() {
        sections.push(warnings_print_table(&warnings));
    }

    PrintReport {
        title: "Diagnostics".to_string(),
        subtitle: print_subtitle("Import quality, detected fields, and warnings.", search),
        generated: print_generated_at(),
        metrics: vec![
            metric(
                "CSV files",
                visible_reports.len().to_string(),
                "stored",
                PrintTone::Normal,
            ),
            metric(
                "Rows seen",
                rows_seen.to_string(),
                "for checks",
                PrintTone::Normal,
            ),
            metric(
                "Imported",
                rows_imported.to_string(),
                trf("{count} skipped", &[("count", rows_skipped.to_string())]),
                PrintTone::Positive,
            ),
            metric(
                "Duplicates",
                data.duplicate_count.to_string(),
                data.dedupe_mode.label(),
                PrintTone::Warning,
            ),
            metric(
                "Unconfigured budgets",
                unconfigured_budget_count.to_string(),
                "Expense transactions with a missing or unknown budget code.",
                PrintTone::Warning,
            ),
            metric(
                "Other categories",
                other_category_count.to_string(),
                "Transactions grouped under OTHER or INC-OTHER.",
                PrintTone::Normal,
            ),
        ],
        sections,
    }
}

fn filtered_warnings(warnings: &[String], search: Option<&SearchFilter>) -> Vec<String> {
    warnings
        .iter()
        .filter(|warning| search.map(|filter| filter.matches(warning)).unwrap_or(true))
        .cloned()
        .collect()
}
