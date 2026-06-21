use super::*;

pub(super) fn transactions_print_report(
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
