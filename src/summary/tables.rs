use super::*;

pub(super) fn category_rows<'a>(
    transactions: impl Iterator<Item = &'a Transaction>,
) -> Vec<Vec<String>> {
    let mut map: HashMap<(String, String), Totals> = HashMap::new();
    for tx in transactions {
        map.entry((tx.category.clone(), tx.budget_code.clone()))
            .or_default()
            .add(tx);
    }
    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort_by(|a, b| {
        b.1.expenses
            .cmp(&a.1.expenses)
            .then_with(|| a.0 .0.cmp(&b.0 .0))
    });
    entries
        .into_iter()
        .map(|((category, budget_code), totals)| {
            vec![
                category,
                budget_code,
                money(totals.expenses),
                money(totals.income),
                signed_money(totals.balance),
                totals.count.to_string(),
            ]
        })
        .collect()
}

pub(super) fn budget_rows(data: &AppData, month: MonthKey) -> Vec<Vec<String>> {
    analytics::budget_usage(&data.transactions, &data.budgets, month)
        .into_iter()
        .map(|budget| {
            vec![
                budget.code,
                budget.category,
                money(budget.budget),
                money(budget.actual),
                signed_money(budget.remaining),
                truncate(&budget.notes, 28),
            ]
        })
        .collect()
}

pub(super) fn truncate(input: &str, max_chars: usize) -> String {
    let count = input.chars().count();
    if count <= max_chars {
        input.to_string()
    } else if max_chars <= 1 {
        "…".to_string()
    } else {
        format!("{}…", input.chars().take(max_chars - 1).collect::<String>())
    }
}
