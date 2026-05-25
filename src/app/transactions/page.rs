use super::*;

pub(in crate::app) fn render_transactions_page(
    data: &AppData,
    ui_handles: &Rc<UiHandles>,
    state: &Rc<RefCell<AppData>>,
) {
    ui::clear_box(&ui_handles.transactions);
    let search = active_search(ui_handles.as_ref());
    let selected_year = transaction_selected_year(data, ui_handles.as_ref());
    let matches = filtered_transactions(&data.transactions, &data.budgets, search.as_ref());
    let subtitle = transaction_page_subtitle(data, &matches, search.as_ref(), selected_year);
    append_page_header(
        &ui_handles.transactions,
        ui_handles.as_ref(),
        "Transactions",
        &subtitle,
        summary::render_transactions(data),
        &data.transactions,
    );

    if let Some(year) = selected_year {
        ui_handles.transactions.append(&year_selector_row(
            &data.available_years,
            year,
            ui_handles,
            state,
        ));
    }
    append_partial_load_notice(&ui_handles.transactions, ui_handles, data);

    if data.transactions.is_empty() {
        if selected_year.is_some() {
            ui_handles.transactions.append(&empty_page(
                "view-list-symbolic",
                "No transactions in this year",
                "Choose another year or import CSV files for this period.",
            ));
        } else {
            ui_handles.transactions.append(&empty_page(
                "view-list-symbolic",
                "No transactions yet",
                "Import CSV files to see recent transactions here.",
            ));
        }
        return;
    }

    if matches.is_empty() {
        ui_handles.transactions.append(&search_empty_page(
            "No transactions found",
            "Adjust your search term or clear the search bar to show all transactions.",
        ));
        return;
    }

    let limit = if search.is_some() { 200 } else { 80 };
    let shown_transactions = matches.iter().copied().take(limit).collect::<Vec<_>>();
    ui_handles
        .transactions
        .append(&transaction_list(&shown_transactions, state, ui_handles));
}

fn transaction_selected_year(data: &AppData, ui_handles: &UiHandles) -> Option<i32> {
    match ui_handles.active_transaction_filter.borrow().as_ref() {
        Some(filter) if filter.is_period_scoped() => filter
            .period_year()
            .or_else(|| selected_year(data, ui_handles)),
        Some(_) => None,
        None => selected_year(data, ui_handles),
    }
}

fn transaction_page_subtitle(
    data: &AppData,
    matches: &[&Transaction],
    search: Option<&SearchFilter>,
    selected_year: Option<i32>,
) -> String {
    if let Some(filter) = search {
        let stats = transaction_result_stats(matches)
            .map(transaction_result_stats_text)
            .unwrap_or_default();
        if let Some(year) = selected_year {
            return trf(
                "{count} of {total} transactions match “{query}” in {year}.{stats}",
                &[
                    ("count", matches.len().to_string()),
                    ("total", data.transactions.len().to_string()),
                    ("query", filter.raw.clone()),
                    ("year", year.to_string()),
                    ("stats", stats),
                ],
            );
        }
        return trf(
            "{count} of {total} transactions match “{query}”.{stats}",
            &[
                ("count", matches.len().to_string()),
                ("total", data.transactions.len().to_string()),
                ("query", filter.raw.clone()),
                ("stats", stats),
            ],
        );
    }

    if let Some(year) = selected_year {
        return trf(
            "{count} transactions in {year}.",
            &[
                ("count", data.transactions.len().to_string()),
                ("year", year.to_string()),
            ],
        );
    }

    tr("Recent transactions as a scannable list. Copy this page for a text export.")
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TransactionResultStats {
    income: Decimal,
    expenses: Decimal,
    balance: Decimal,
    average: Decimal,
    largest_income: Option<Decimal>,
    largest_expense: Option<Decimal>,
    modal: Option<Decimal>,
}

fn transaction_result_stats(transactions: &[&Transaction]) -> Option<TransactionResultStats> {
    let count = transactions.len();
    if count == 0 {
        return None;
    }

    let mut totals = analytics::Totals::default();
    let mut largest_income = None;
    let mut largest_expense = None;
    for transaction in transactions {
        totals.add(transaction);
        if transaction.amount > Decimal::ZERO {
            largest_income = Some(max_optional(largest_income, transaction.amount));
        } else if transaction.amount < Decimal::ZERO {
            largest_expense = Some(max_optional(largest_expense, -transaction.amount));
        }
    }

    Some(TransactionResultStats {
        income: totals.income,
        expenses: totals.expenses,
        balance: totals.balance,
        average: totals.balance / Decimal::from(count as u64),
        largest_income,
        largest_expense,
        modal: modal_transaction_amount(transactions),
    })
}

fn max_optional(current: Option<Decimal>, next: Decimal) -> Decimal {
    current.map_or(next, |current| current.max(next))
}

fn modal_transaction_amount(transactions: &[&Transaction]) -> Option<Decimal> {
    let mut counts = std::collections::BTreeMap::new();
    for transaction in transactions {
        *counts.entry(transaction.amount).or_insert(0usize) += 1;
    }
    let max_count = counts.values().copied().max().unwrap_or(0);
    if max_count < 2 {
        return None;
    }

    transactions
        .iter()
        .map(|transaction| transaction.amount)
        .find(|amount| counts.get(amount).copied() == Some(max_count))
}

fn transaction_result_stats_text(stats: TransactionResultStats) -> String {
    trf(
        " Balance {balance}; income {income}; expenses {expenses}; avg {average}; largest income {largest_income}; largest expense {largest_expense}; modal {modal}.",
        &[
            ("balance", signed_money(stats.balance)),
            ("income", money(stats.income)),
            ("expenses", money(stats.expenses)),
            ("average", signed_money(stats.average)),
            (
                "largest_income",
                stats.largest_income.map(money).unwrap_or_else(|| tr("none")),
            ),
            (
                "largest_expense",
                stats.largest_expense.map(money).unwrap_or_else(|| tr("none")),
            ),
            (
                "modal",
                stats.modal.map(signed_money).unwrap_or_else(|| tr("none")),
            ),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn tx(amount: i64) -> Transaction {
        Transaction {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            amount: Decimal::new(amount, 0),
            description: String::new(),
            counterparty: String::new(),
            tags: String::new(),
            account: String::new(),
            transaction_id: String::new(),
            currency: "EUR".to_string(),
            source_file: String::new(),
            source_row: 0,
            category: String::new(),
            budget_code: String::new(),
            notes: String::new(),
            strict_key: String::new(),
            loose_key: String::new(),
            rule_match: None,
        }
    }

    #[test]
    fn transaction_result_stats_include_balance_and_useful_amounts() {
        let transactions = [tx(10), tx(-5), tx(-5)];
        let refs = transactions.iter().collect::<Vec<_>>();

        assert_eq!(
            transaction_result_stats(&refs),
            Some(TransactionResultStats {
                income: Decimal::new(10, 0),
                expenses: Decimal::new(10, 0),
                balance: Decimal::ZERO,
                average: Decimal::ZERO,
                largest_income: Some(Decimal::new(10, 0)),
                largest_expense: Some(Decimal::new(5, 0)),
                modal: Some(Decimal::new(-5, 0)),
            })
        );
    }

    #[test]
    fn transaction_result_stats_have_no_modal_for_unique_amounts() {
        let transactions = [tx(10), tx(-5), tx(20)];
        let refs = transactions.iter().collect::<Vec<_>>();

        assert_eq!(transaction_result_stats(&refs).unwrap().modal, None);
    }

    #[test]
    fn transaction_result_stats_have_no_largest_values_for_zero_only_results() {
        let transactions = [tx(0), tx(0)];
        let refs = transactions.iter().collect::<Vec<_>>();
        let stats = transaction_result_stats(&refs).unwrap();

        assert_eq!(stats.largest_income, None);
        assert_eq!(stats.largest_expense, None);
    }

    #[test]
    fn transaction_result_stats_are_absent_without_results() {
        assert_eq!(transaction_result_stats(&[]), None);
    }
}
