use super::*;

pub(super) fn budget_print_report(
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
