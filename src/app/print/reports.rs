use super::*;

pub(in crate::app) fn current_print_report(data: &AppData, ui: &UiHandles) -> PrintReport {
    let runtime_data = data_with_fake_transactions(data.clone(), ui.fake_transactions.list());
    let filtered = filtered_app_data(&runtime_data, ui);
    let print_data = filtered.as_ref().unwrap_or(&runtime_data);
    let search = active_search(ui);
    match ui.stack.visible_child_name().as_deref() {
        Some("overview") => overview_print_report(print_data, ui, search.as_ref()),
        Some("transactions") => transactions_print_report(print_data, search.as_ref()),
        Some("debug") => diagnostics_print_report(print_data, search.as_ref()),
        _ => budget_print_report(print_data, ui, search.as_ref()),
    }
}

fn print_subtitle(default: &str, search: Option<&SearchFilter>) -> String {
    search
        .map(|filter| {
            trf(
                "Filtered by “{query}”. {description}",
                &[
                    ("query", filter.raw.clone()),
                    ("description", default.to_string()),
                ],
            )
        })
        .unwrap_or_else(|| default.to_string())
}

pub(in crate::app) fn overview_print_report(
    data: &AppData,
    ui: &UiHandles,
    search: Option<&SearchFilter>,
) -> PrintReport {
    let dashboard = analytics::dashboard(data);
    let latest = dashboard
        .latest_month
        .map(ui::month_label)
        .unwrap_or_else(|| "-".to_string());
    let selected_year = ui
        .selected_year
        .get()
        .or_else(|| data.default_month.map(|month| month.year))
        .or_else(|| data.available_years.last().copied());
    let comparison = if ui.compare_categories_previous_period.get() {
        ComparisonMode::WithPrevious
    } else {
        ComparisonMode::CurrentOnly
    };

    let mut sections = Vec::new();
    if data.transactions.is_empty() {
        sections.push(PrintSection::Paragraph {
            title: "No transactions yet".to_string(),
            body: "Choose CSV files or drop bank files onto the window to print monthly and yearly overviews.".to_string(),
        });
    } else {
        sections.push(monthly_print_table(&dashboard.monthly));
        if let Some(year) = selected_year {
            let warnings =
                attention_warning_messages(&annual_budget_attention_warnings(data, year));
            if !warnings.is_empty() {
                sections.push(warnings_print_table(&warnings));
            }
            if comparison.includes_previous() {
                if let Some(year_comparison) =
                    analytics::year_comparison(&data.transactions, &data.budgets, year)
                {
                    sections.push(year_comparison_print_table(&year_comparison));
                }
            }
            sections.push(annual_budgets_print_table(data, year, comparison));
            if comparison.includes_previous() {
                sections.push(annual_categories_print_table(data, year, comparison));
            }
        }
    }

    PrintReport {
        title: "Overview".to_string(),
        subtitle: print_subtitle("Monthly trend, yearly comparison, and budget room.", search),
        generated: print_generated_at(),
        metrics: vec![
            metric(
                "Transactions",
                data.transactions.len().to_string(),
                "imported",
                PrintTone::Normal,
            ),
            metric(
                "Latest month",
                signed_money(dashboard.latest_totals.balance),
                latest,
                tone_for_amount(dashboard.latest_totals.balance),
            ),
            metric(
                "Total balance",
                signed_money(dashboard.all_totals.balance),
                "all CSV files",
                tone_for_amount(dashboard.all_totals.balance),
            ),
            metric(
                "Active rules",
                data.rules_count.to_string(),
                trf(
                    "duplicate filtering {state}",
                    &[("state", tr(data.dedupe_mode.label()))],
                ),
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

pub(in crate::app) fn budget_print_report(
    data: &AppData,
    ui: &UiHandles,
    search: Option<&SearchFilter>,
) -> PrintReport {
    let selected_month = selected_budget_month(data, ui);
    let month_label = selected_month
        .map(ui::month_label)
        .unwrap_or_else(|| "no period".to_string());
    let totals = selected_month
        .map(|month| totals_for_month(data, month))
        .unwrap_or_default();

    let mut sections = Vec::new();
    if let Some(month) = selected_month {
        let warnings = attention_warning_messages(&monthly_budget_attention_warnings(data, month));
        if !warnings.is_empty() {
            sections.push(warnings_print_table(&warnings));
        }
        sections.push(budget_usage_print_table(data, month));
        sections.push(month_categories_print_table(data, month));
        sections.push(month_transactions_print_table(data, month));
    } else {
        sections.push(PrintSection::Paragraph {
            title: "No period".to_string(),
            body: "Import CSV files to print budgets and categories.".to_string(),
        });
    }

    PrintReport {
        title: "Budget".to_string(),
        subtitle: print_subtitle(
            &trf(
                "Budget room and categories for {month}.",
                &[("month", month_label.clone())],
            ),
            search,
        ),
        generated: print_generated_at(),
        metrics: vec![
            metric(
                "Expenses",
                money(totals.expenses),
                &month_label,
                PrintTone::Negative,
            ),
            metric(
                "Income",
                money(totals.income),
                &month_label,
                PrintTone::Positive,
            ),
            metric(
                "Balance",
                signed_money(totals.balance),
                &month_label,
                tone_for_amount(totals.balance),
            ),
            metric(
                "Budget codes",
                data.budgets.len().to_string(),
                "configured",
                PrintTone::Normal,
            ),
        ],
        sections,
    }
}

pub(in crate::app) fn transactions_print_report(
    data: &AppData,
    search: Option<&SearchFilter>,
) -> PrintReport {
    let totals = analytics::dashboard(data).all_totals;
    PrintReport {
        title: "Transactions".to_string(),
        subtitle: print_subtitle(
            "Latest transactions with category, budget code, and description.",
            search,
        ),
        generated: print_generated_at(),
        metrics: vec![
            metric(
                "Transactions",
                totals.count.to_string(),
                "total",
                PrintTone::Normal,
            ),
            metric("Income", money(totals.income), "total", PrintTone::Positive),
            metric(
                "Expenses",
                money(totals.expenses),
                "total",
                PrintTone::Negative,
            ),
            metric(
                "Balance",
                signed_money(totals.balance),
                "total",
                tone_for_amount(totals.balance),
            ),
        ],
        sections: vec![transactions_print_table(&data.transactions, search)],
    }
}

pub(in crate::app) fn diagnostics_print_report(
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
