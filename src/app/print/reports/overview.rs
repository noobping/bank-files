use super::*;

pub(super) fn overview_print_report(
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
