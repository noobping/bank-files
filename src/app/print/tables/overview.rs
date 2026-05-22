use super::*;

pub(in crate::app) fn monthly_print_table(months: &[analytics::MonthSummary]) -> PrintSection {
    PrintSection::Table {
        title: "Months".to_string(),
        subtitle: "Latest 24 months with income, expenses, and balance.".to_string(),
        columns: columns(&[
            ("Period", 1.1, PrintAlign::Left),
            ("Income", 1.0, PrintAlign::Right),
            ("Expenses", 1.0, PrintAlign::Right),
            ("Balance", 1.0, PrintAlign::Right),
            ("Tx", 0.45, PrintAlign::Right),
        ]),
        rows: months
            .iter()
            .map(|month| {
                vec![
                    cell(ui::month_label(month.month)),
                    tone_cell(money(month.totals.income), PrintTone::Positive),
                    tone_cell(money(month.totals.expenses), PrintTone::Negative),
                    tone_cell(
                        signed_money(month.totals.balance),
                        tone_for_amount(month.totals.balance),
                    ),
                    cell(month.totals.count.to_string()),
                ]
            })
            .collect(),
    }
}

pub(in crate::app) fn year_comparison_print_table(
    comparison: &analytics::YearComparison,
) -> PrintSection {
    PrintSection::Table {
        title: "Year Comparison".to_string(),
        subtitle: trf(
            "{year} compared with {previous_year}.",
            &[
                ("year", comparison.year.to_string()),
                ("previous_year", comparison.previous_year.to_string()),
            ],
        ),
        columns: columns(&[
            ("Metric", 1.4, PrintAlign::Left),
            (
                &comparison.previous_year.to_string(),
                0.9,
                PrintAlign::Right,
            ),
            (&comparison.year.to_string(), 0.9, PrintAlign::Right),
            ("Difference", 0.9, PrintAlign::Right),
        ]),
        rows: vec![
            vec![
                cell("Income"),
                tone_cell(money(comparison.previous.income), PrintTone::Positive),
                tone_cell(money(comparison.current.income), PrintTone::Positive),
                tone_cell(
                    signed_money(comparison.income_delta),
                    tone_for_amount(comparison.income_delta),
                ),
            ],
            vec![
                cell("Expenses"),
                tone_cell(money(comparison.previous.expenses), PrintTone::Negative),
                tone_cell(money(comparison.current.expenses), PrintTone::Negative),
                tone_cell(
                    signed_money(comparison.expense_delta),
                    if comparison.expense_delta > Decimal::ZERO {
                        PrintTone::Negative
                    } else {
                        PrintTone::Positive
                    },
                ),
            ],
            vec![
                cell("Balance"),
                tone_cell(
                    signed_money(comparison.previous.balance),
                    tone_for_amount(comparison.previous.balance),
                ),
                tone_cell(
                    signed_money(comparison.current.balance),
                    tone_for_amount(comparison.current.balance),
                ),
                tone_cell(
                    signed_money(comparison.balance_delta),
                    tone_for_amount(comparison.balance_delta),
                ),
            ],
        ],
    }
}

pub(in crate::app) fn annual_budgets_print_table(
    data: &AppData,
    year: i32,
    comparison: ComparisonMode,
) -> PrintSection {
    PrintSection::Table {
        title: "Annual Budgets".to_string(),
        subtitle: if comparison.includes_previous() {
            trf(
                "Fixed monthly budgets are annualized; yearly budgets are used as-is; percentage budgets use yearly income. Gray values compare with {previous_year}.",
                &[("previous_year", (year - 1).to_string())],
            )
        } else {
            tr("Fixed monthly budgets are annualized; yearly budgets are used as-is; percentage budgets use yearly income.")
        },
        columns: annual_budget_columns(year, comparison),
        rows: analytics::annual_budget_usage(&data.transactions, &data.budgets, year, comparison)
            .into_iter()
            .take(14)
            .map(|budget| annual_budget_print_row(budget, comparison))
            .collect(),
    }
}

fn annual_budget_columns(year: i32, comparison: ComparisonMode) -> Vec<PrintColumn> {
    let mut cols = vec![
        PrintColumn {
            title: "Code".to_string(),
            width: 0.55,
            align: PrintAlign::Left,
        },
        PrintColumn {
            title: "Category".to_string(),
            width: 1.4,
            align: PrintAlign::Left,
        },
        PrintColumn {
            title: "Annual budget".to_string(),
            width: 0.9,
            align: PrintAlign::Right,
        },
        PrintColumn {
            title: year.to_string(),
            width: 0.8,
            align: PrintAlign::Right,
        },
    ];
    if comparison.includes_previous() {
        cols.push(PrintColumn {
            title: (year - 1).to_string(),
            width: 0.8,
            align: PrintAlign::Right,
        });
    }
    cols.push(PrintColumn {
        title: "Room".to_string(),
        width: 0.85,
        align: PrintAlign::Right,
    });
    cols
}

fn annual_budget_print_row(
    budget: analytics::AnnualBudgetUsage,
    comparison: ComparisonMode,
) -> Vec<PrintCell> {
    let mut row = vec![
        cell(budget.code),
        cell(truncate(&budget.category, 32)),
        cell(planned_budget_label(budget.budget, &budget.budget_basis)),
        tone_cell(money(budget.actual), PrintTone::Negative),
    ];
    if comparison.includes_previous() {
        row.push(tone_cell(
            money(budget.previous_actual.unwrap_or(Decimal::ZERO)),
            PrintTone::Muted,
        ));
    }
    row.push(tone_cell(
        signed_money(budget.remaining),
        tone_for_amount(budget.remaining),
    ));
    row
}
