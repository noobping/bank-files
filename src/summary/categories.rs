use super::*;

pub fn render_categories(data: &AppData) -> String {
    if data.transactions.is_empty() {
        let mut out = String::new();
        out.push_str("Categories and budget codes\n");
        out.push_str("===========================\n\n");
        out.push_str("No transactions yet.\n");
        return out;
    }

    let Some(latest_month) = data.transactions.iter().map(Transaction::month_key).max() else {
        let mut out = String::new();
        out.push_str("Categories and budget codes\n");
        out.push_str("===========================\n\n");
        out.push_str("No transactions yet.\n");
        return out;
    };
    render_categories_for_month(data, latest_month)
}

pub fn render_categories_for_month(data: &AppData, month: MonthKey) -> String {
    let mut out = String::new();
    out.push_str("Categories and budget codes\n");
    out.push_str("===========================\n\n");

    if data.transactions.is_empty() {
        out.push_str("No transactions yet.\n");
        return out;
    }

    let selected_year = month.year;

    out.push_str(&format!("Selected month: {}\n\n", month_label(month)));
    out.push_str("Expenses by category - selected month\n-------------------------------------\n");
    let month_rows = category_rows(
        data.transactions
            .iter()
            .filter(|tx| tx.month_key() == month),
    );
    out.push_str(&pad_table(
        &[
            "Category",
            "Budgetcode",
            "Expenses",
            "Income",
            "Balance",
            "Tx",
        ],
        &month_rows,
    ));

    out.push_str(&format!(
        "\nExpenses by category - {selected_year}\n-----------------------------------\n"
    ));
    let year_rows = category_rows(
        data.transactions
            .iter()
            .filter(|tx| tx.year() == selected_year),
    );
    out.push_str(&pad_table(
        &[
            "Category",
            "Budgetcode",
            "Expenses",
            "Income",
            "Balance",
            "Tx",
        ],
        &year_rows,
    ));

    out.push_str("\nBudget check - selected month\n-----------------------------\n");
    let budget_rows = budget_rows(data, month);
    if budget_rows.is_empty() {
        out.push_str("No budget codes found. Open Rules to edit budgetcodes.csv.\n");
    } else {
        out.push_str(&pad_table(
            &[
                "Code",
                "Category",
                "Budget",
                "Actual",
                "Remaining/over",
                "Note",
            ],
            &budget_rows,
        ));
    }

    out
}

pub fn render_transactions(data: &AppData) -> String {
    let mut out = String::new();
    out.push_str("Transactions\n");
    out.push_str("===========\n\n");
    out.push_str("Latest 500 transactions, sorted by date.\n\n");

    let mut rows = Vec::new();
    for tx in data.transactions.iter().take(500) {
        rows.push(vec![
            tx.date.to_string(),
            signed_money(tx.amount),
            truncate(&tx.category, 24),
            truncate(&tx.budget_code, 10),
            truncate(&tx.counterparty, 28),
            truncate(&tx.tags, 18),
            truncate(&tx.description, 46),
            truncate(&tx.source_file, 22),
        ]);
    }
    out.push_str(&pad_table(
        &[
            "Date",
            "Amount",
            "Category",
            "Budget",
            "Counterparty",
            "Tags",
            "Description",
            "Source",
        ],
        &rows,
    ));
    out
}
