use super::*;

pub(in crate::app) fn annual_categories_print_table(
    data: &AppData,
    year: i32,
    comparison: ComparisonMode,
) -> PrintSection {
    PrintSection::Table {
        title: "Annual Spending".to_string(),
        subtitle: trf(
            "Largest annual expenses by category in {year}.",
            &[("year", year.to_string())],
        ),
        columns: columns(&[
            ("Category", 1.7, PrintAlign::Left),
            ("Budget", 0.65, PrintAlign::Left),
            (&year.to_string(), 0.85, PrintAlign::Right),
            (&(year - 1).to_string(), 0.85, PrintAlign::Right),
            ("Difference", 0.85, PrintAlign::Right),
        ]),
        rows: analytics::category_totals_for_year_comparison(
            &data.transactions,
            &data.budgets,
            year,
            14,
            comparison,
        )
        .into_iter()
        .map(|category| {
            let delta = category.current.expenses
                - category
                    .previous
                    .as_ref()
                    .map(|totals| totals.expenses)
                    .unwrap_or(Decimal::ZERO);
            vec![
                cell(truncate(&category.category, 34)),
                cell(category.budget_code),
                tone_cell(money(category.current.expenses), PrintTone::Negative),
                tone_cell(
                    money(
                        category
                            .previous
                            .as_ref()
                            .map(|totals| totals.expenses)
                            .unwrap_or(Decimal::ZERO),
                    ),
                    PrintTone::Muted,
                ),
                tone_cell(
                    signed_money(delta),
                    if delta > Decimal::ZERO {
                        PrintTone::Negative
                    } else {
                        PrintTone::Positive
                    },
                ),
            ]
        })
        .collect(),
    }
}

pub(in crate::app) fn budget_usage_print_table(data: &AppData, month: MonthKey) -> PrintSection {
    PrintSection::Table {
        title: "Budget Room".to_string(),
        subtitle: trf(
            "Budgets for {month}. Yearly-only budgets use remaining annual room.",
            &[("month", ui::month_label(month))],
        ),
        columns: columns(&[
            ("Code", 0.55, PrintAlign::Left),
            ("Category", 1.45, PrintAlign::Left),
            ("Budget", 0.85, PrintAlign::Right),
            ("Actual", 0.85, PrintAlign::Right),
            ("Room", 0.85, PrintAlign::Right),
        ]),
        rows: analytics::budget_usage(&data.transactions, &data.budgets, month)
            .into_iter()
            .map(|budget| {
                vec![
                    cell(budget.code),
                    cell(truncate(&budget.category, 34)),
                    cell(planned_budget_label(budget.budget, &budget.budget_basis)),
                    tone_cell(money(budget.actual), PrintTone::Negative),
                    tone_cell(
                        signed_money(budget.remaining),
                        tone_for_amount(budget.remaining),
                    ),
                ]
            })
            .collect(),
    }
}

pub(in crate::app) fn month_categories_print_table(
    data: &AppData,
    month: MonthKey,
) -> PrintSection {
    PrintSection::Table {
        title: "Monthly Spending".to_string(),
        subtitle: trf(
            "Largest expenses in {month}.",
            &[("month", ui::month_label(month))],
        ),
        columns: columns(&[
            ("Category", 1.65, PrintAlign::Left),
            ("Budget", 0.65, PrintAlign::Left),
            ("Expenses", 0.85, PrintAlign::Right),
            ("Income", 0.85, PrintAlign::Right),
            ("Balance", 0.85, PrintAlign::Right),
            ("Tx", 0.35, PrintAlign::Right),
        ]),
        rows: analytics::category_totals_for_month(&data.transactions, &data.budgets, month, 16)
            .into_iter()
            .map(|category| {
                vec![
                    cell(truncate(&category.category, 32)),
                    cell(category.budget_code),
                    tone_cell(money(category.totals.expenses), PrintTone::Negative),
                    tone_cell(money(category.totals.income), PrintTone::Positive),
                    tone_cell(
                        signed_money(category.totals.balance),
                        tone_for_amount(category.totals.balance),
                    ),
                    cell(category.totals.count.to_string()),
                ]
            })
            .collect(),
    }
}

pub(in crate::app) fn month_transactions_print_table(
    data: &AppData,
    month: MonthKey,
) -> PrintSection {
    PrintSection::Table {
        title: "Transactions in Period".to_string(),
        subtitle: "Latest 40 transactions for this month.".to_string(),
        columns: transaction_print_columns(),
        rows: data
            .transactions
            .iter()
            .filter(|tx| tx.month_key() == month)
            .take(40)
            .map(transaction_print_row)
            .collect(),
    }
}

pub(in crate::app) fn transactions_print_table(
    transactions: &[Transaction],
    search: Option<&SearchFilter>,
) -> PrintSection {
    PrintSection::Table {
        title: "Transaction List".to_string(),
        subtitle: search
            .map(|filter| {
                trf(
                    "Latest 200 transactions matching “{query}”.",
                    &[("query", filter.raw.clone())],
                )
            })
            .unwrap_or_else(|| "Latest 200 transactions.".to_string()),
        columns: transaction_print_columns(),
        rows: transactions
            .iter()
            .take(200)
            .map(transaction_print_row)
            .collect(),
    }
}

pub(in crate::app) fn diagnostic_files_print_table(
    reports: &[&crate::model::ImportReport],
) -> PrintSection {
    PrintSection::Table {
        title: "CSV Files".to_string(),
        subtitle: "Import quality per file.".to_string(),
        columns: columns(&[
            ("File", 1.7, PrintAlign::Left),
            ("Delimiter", 0.7, PrintAlign::Left),
            ("Seen", 0.55, PrintAlign::Right),
            ("Import", 0.55, PrintAlign::Right),
            ("Skipped", 0.55, PrintAlign::Right),
            ("Fields", 1.6, PrintAlign::Left),
        ]),
        rows: reports
            .iter()
            .map(|report| {
                let report = *report;
                let name = report
                    .source
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| report.source.display().to_string());
                let fields = import_report_field_text(report, ", ");
                vec![
                    cell(truncate(&name, 34)),
                    cell(delimiter_label(report.delimiter)),
                    cell(report.rows_seen.to_string()),
                    tone_cell(report.rows_imported.to_string(), PrintTone::Positive),
                    tone_cell(
                        report.rows_skipped.to_string(),
                        if report.rows_skipped > 0 {
                            PrintTone::Warning
                        } else {
                            PrintTone::Muted
                        },
                    ),
                    cell(truncate(&fields, 42)),
                ]
            })
            .collect(),
    }
}

pub(in crate::app) fn warnings_print_table(warnings: &[String]) -> PrintSection {
    PrintSection::Table {
        title: "Warnings".to_string(),
        subtitle: "Messages that need attention.".to_string(),
        columns: columns(&[("Message", 1.0, PrintAlign::Left)]),
        rows: warnings
            .iter()
            .map(|warning| vec![tone_cell(truncate(warning, 120), PrintTone::Warning)])
            .collect(),
    }
}
