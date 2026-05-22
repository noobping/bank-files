use super::*;
use crate::app::transactions::transaction_search_text;
use crate::model::{BudgetCode, MonthKey};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum AppPage {
    Overview,
    Budget,
    Transactions,
    Diagnostics,
}

impl AppPage {
    fn from_stack_name(name: Option<&str>) -> Self {
        match name {
            Some("categories") => Self::Budget,
            Some("transactions") => Self::Transactions,
            Some("debug") => Self::Diagnostics,
            _ => Self::Overview,
        }
    }
}

pub(in crate::app) fn current_page(ui: &UiHandles) -> AppPage {
    AppPage::from_stack_name(ui.stack.visible_child_name().as_deref())
}

pub(in crate::app) fn filtered_app_data(data: &AppData, ui: &UiHandles) -> Option<AppData> {
    let search = active_search(ui);
    let transaction_filter = ui.active_transaction_filter.borrow();
    page_filtered_app_data(
        data,
        current_page(ui),
        effective_hide_canceled_transactions(
            ui.show_predictions.get(),
            ui.hide_canceled_transactions.get(),
        ),
        search.as_ref(),
        transaction_filter.as_ref(),
    )
}

pub(in crate::app) fn page_data_for_render(
    data: &AppData,
    page: AppPage,
    hide_canceled: bool,
    transaction_filter: Option<&TransactionFilter>,
) -> Option<AppData> {
    page_filtered_app_data(data, page, hide_canceled, None, transaction_filter)
}

fn page_filtered_app_data(
    data: &AppData,
    page: AppPage,
    hide_canceled: bool,
    search: Option<&SearchFilter>,
    transaction_filter: Option<&TransactionFilter>,
) -> Option<AppData> {
    if search.is_none()
        && !page_hides_canceled_transactions(page, hide_canceled, transaction_filter)
    {
        return None;
    }

    let mut transactions = data.transactions.clone();
    if page_hides_canceled_transactions(page, hide_canceled, transaction_filter) {
        transactions = analytics::transactions_without_canceled_patterns(&transactions);
    }
    if let Some(search) = search {
        transactions = filtered_transactions(&transactions, &data.budgets, Some(search))
            .into_iter()
            .cloned()
            .collect();
    }

    if transactions.len() == data.transactions.len() {
        return None;
    }

    let mut filtered = data.clone();
    filtered.transactions = transactions;
    Some(filtered)
}

fn page_hides_canceled_transactions(
    page: AppPage,
    hide_canceled: bool,
    transaction_filter: Option<&TransactionFilter>,
) -> bool {
    hide_canceled
        && page != AppPage::Diagnostics
        && !(page == AppPage::Transactions
            && transaction_filter
                .map(TransactionFilter::includes_diagnostic_hidden_rows)
                .unwrap_or(false))
}

pub(in crate::app) fn connect_search(state: &Rc<RefCell<AppData>>, ui: &Rc<UiHandles>) {
    let state_for_search = Rc::clone(state);
    let ui_for_search = Rc::clone(ui);
    ui.search_entry.connect_search_changed(move |entry| {
        let query = entry.text().trim().to_string();
        {
            let mut current = ui_for_search.search_query.borrow_mut();
            if *current == query {
                return;
            }
            *current = query.clone();
        }
        *ui_for_search.active_transaction_filter.borrow_mut() =
            TransactionFilter::from_query(&query);

        render_views(
            &state_for_search.borrow(),
            &ui_for_search,
            &state_for_search,
        );
        if query.is_empty() {
            show_status(&ui_for_search, "Filter cleared. All items are shown.");
        } else {
            show_status(
                &ui_for_search,
                &trf(
                    "Filter active: “{query}”. Clear the search text to show everything.",
                    &[("query", query)],
                ),
            );
        }
    });

    let search_bar_for_stop = ui.search_bar.clone();
    ui.search_entry.connect_stop_search(move |entry| {
        entry.set_text("");
        search_bar_for_stop.set_search_mode(false);
    });

    let search_entry_for_close = ui.search_entry.clone();
    ui.search_bar
        .connect_search_mode_enabled_notify(move |search_bar| {
            if !search_bar.is_search_mode() && !search_entry_for_close.text().is_empty() {
                search_entry_for_close.set_text("");
            }
        });
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum TransactionAmountFilter {
    Income,
    Expense,
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::app) enum TransactionFilter {
    All,
    UnconfiguredBudgets,
    OtherCategories,
    CategoryForYear {
        category: String,
        year: i32,
    },
    Pattern(analytics::TransactionPattern),
    Scoped {
        budget_code: Option<String>,
        year: Option<i32>,
        month: Option<MonthKey>,
        amount: Option<TransactionAmountFilter>,
    },
}

impl TransactionFilter {
    pub(in crate::app) fn all() -> Self {
        Self::All
    }

    pub(in crate::app) fn year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: None,
        }
    }

    pub(in crate::app) fn month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: None,
        }
    }

    pub(in crate::app) fn income_for_year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: Some(TransactionAmountFilter::Income),
        }
    }

    pub(in crate::app) fn expenses_for_year(year: i32) -> Self {
        Self::Scoped {
            budget_code: None,
            year: Some(year),
            month: None,
            amount: Some(TransactionAmountFilter::Expense),
        }
    }

    pub(in crate::app) fn category_for_year(category: impl Into<String>, year: i32) -> Self {
        Self::CategoryForYear {
            category: category.into(),
            year,
        }
    }

    pub(in crate::app) fn income_for_month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: Some(TransactionAmountFilter::Income),
        }
    }

    pub(in crate::app) fn expenses_for_month(month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: None,
            year: None,
            month: Some(month),
            amount: Some(TransactionAmountFilter::Expense),
        }
    }

    pub(in crate::app) fn budget_for_year(code: impl Into<String>, year: i32) -> Self {
        Self::Scoped {
            budget_code: Some(code.into()),
            year: Some(year),
            month: None,
            amount: None,
        }
    }

    pub(in crate::app) fn budget_for_month(code: impl Into<String>, month: MonthKey) -> Self {
        Self::Scoped {
            budget_code: Some(code.into()),
            year: None,
            month: Some(month),
            amount: None,
        }
    }

    pub(in crate::app) fn with_year(&self, year: i32) -> Option<Self> {
        match self {
            Self::CategoryForYear { category, .. } => Some(Self::category_for_year(category, year)),
            Self::Scoped {
                budget_code,
                month,
                amount,
                ..
            } => Some(Self::Scoped {
                budget_code: budget_code.clone(),
                year: month.is_none().then_some(year),
                month: month.map(|month| MonthKey::new(year, month.month)),
                amount: *amount,
            }),
            Self::All | Self::UnconfiguredBudgets | Self::OtherCategories | Self::Pattern(_) => {
                None
            }
        }
    }

    pub(in crate::app) fn is_period_scoped(&self) -> bool {
        matches!(self, Self::CategoryForYear { .. } | Self::Scoped { .. })
    }

    fn includes_diagnostic_hidden_rows(&self) -> bool {
        matches!(
            self,
            Self::UnconfiguredBudgets | Self::OtherCategories | Self::Pattern(_)
        )
    }

    pub(in crate::app) fn period_year(&self) -> Option<i32> {
        match self {
            Self::CategoryForYear { year, .. } => Some(*year),
            Self::Scoped { year, month, .. } => year.or_else(|| month.map(|month| month.year)),
            Self::All | Self::UnconfiguredBudgets | Self::OtherCategories | Self::Pattern(_) => {
                None
            }
        }
    }

    pub(in crate::app) fn label(&self) -> &'static str {
        match self {
            Self::All => "All transactions",
            Self::UnconfiguredBudgets => "Unconfigured budgets",
            Self::OtherCategories => "Other categories",
            Self::CategoryForYear { .. } => "Category transactions",
            Self::Pattern(_) => "Transaction pattern",
            Self::Scoped { .. } => "Transactions",
        }
    }

    pub(in crate::app) fn description(&self) -> &'static str {
        match self {
            Self::All => "All transactions",
            Self::UnconfiguredBudgets => {
                "Expense transactions with a missing or unknown budget code."
            }
            Self::OtherCategories => "Transactions grouped under OTHER or INC-OTHER.",
            Self::CategoryForYear { .. } => "Transactions for this category and year.",
            Self::Pattern(_) => "Transactions matching the detected pattern.",
            Self::Scoped { .. } => "Filtered transactions",
        }
    }

    pub(in crate::app) fn query(&self) -> String {
        match self {
            Self::All => String::new(),
            Self::UnconfiguredBudgets | Self::OtherCategories => tr(self.label()),
            Self::CategoryForYear { category, year } => {
                format!("category:{} year:{year}", encode_filter_value(category))
            }
            Self::Pattern(pattern) => pattern.label.clone(),
            Self::Scoped {
                budget_code,
                year,
                month,
                amount,
            } => {
                let mut tokens = Vec::new();
                if let Some(code) = budget_code {
                    tokens.push(format!("budget:{code}"));
                }
                if let Some(month) = month {
                    tokens.push(format!("month:{month}"));
                } else if let Some(year) = year {
                    tokens.push(format!("year:{year}"));
                }
                if let Some(amount) = amount {
                    tokens.push(match amount {
                        TransactionAmountFilter::Income => "amount:income".to_string(),
                        TransactionAmountFilter::Expense => "amount:expense".to_string(),
                    });
                }
                tokens.join(" ")
            }
        }
    }

    pub(in crate::app) fn from_query(text: &str) -> Option<Self> {
        let normalized = text.trim().to_lowercase();
        for filter in [Self::UnconfiguredBudgets, Self::OtherCategories] {
            if normalized == filter.label().to_lowercase()
                || normalized == tr(filter.label()).to_lowercase()
            {
                return Some(filter);
            }
        }

        let mut budget_code = None;
        let mut year = None;
        let mut month = None;
        let mut amount = None;
        let mut category = None;
        let mut recognized = false;

        for token in text.split_whitespace() {
            let Some((key, value)) = token.split_once(':') else {
                continue;
            };
            let value = value.trim();
            match key.trim().to_lowercase().as_str() {
                "budget" | "code" if !value.is_empty() => {
                    budget_code = Some(value.to_string());
                    recognized = true;
                }
                "year" => {
                    if let Ok(parsed) = value.parse::<i32>() {
                        year = Some(parsed);
                        recognized = true;
                    }
                }
                "month" => {
                    if let Some(parsed) = parse_month_filter(value) {
                        month = Some(parsed);
                        recognized = true;
                    }
                }
                "category" | "cat" if !value.is_empty() => {
                    category = Some(decode_filter_value(value));
                }
                "amount" | "type" => match value.to_lowercase().as_str() {
                    "income" | "in" => {
                        amount = Some(TransactionAmountFilter::Income);
                        recognized = true;
                    }
                    "expense" | "expenses" | "out" => {
                        amount = Some(TransactionAmountFilter::Expense);
                        recognized = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if let (Some(category), Some(year)) = (category, year) {
            return Some(Self::CategoryForYear { category, year });
        }

        recognized.then_some(Self::Scoped {
            budget_code,
            year,
            month,
            amount,
        })
    }

    pub(in crate::app) fn matches(
        &self,
        tx: &crate::model::Transaction,
        budgets: &[BudgetCode],
    ) -> bool {
        match self {
            Self::All => true,
            Self::UnconfiguredBudgets => {
                analytics::transaction_has_unconfigured_expense_budget(tx, budgets)
            }
            Self::OtherCategories => matches!(tx.budget_code.trim(), "OTHER" | "INC-OTHER"),
            Self::CategoryForYear { category, year } => {
                tx.year() == *year && transaction_category_label(tx) == category.trim()
            }
            Self::Pattern(pattern) => analytics::transaction_matches_pattern(tx, pattern),
            Self::Scoped {
                budget_code,
                year,
                month,
                amount,
            } => {
                if let Some(month) = month {
                    if tx.month_key() != *month {
                        return false;
                    }
                } else if let Some(year) = year {
                    if tx.year() != *year {
                        return false;
                    }
                }
                if let Some(code) = budget_code {
                    if tx.budget_code.trim() != code.trim() {
                        return false;
                    }
                }
                match amount {
                    Some(TransactionAmountFilter::Income) => {
                        !analytics::transaction_is_transfer(tx, budgets)
                            && tx.amount > rust_decimal::Decimal::ZERO
                    }
                    Some(TransactionAmountFilter::Expense) => {
                        !analytics::transaction_is_transfer(tx, budgets)
                            && tx.amount < rust_decimal::Decimal::ZERO
                    }
                    None => true,
                }
            }
        }
    }
}

fn transaction_category_label(tx: &Transaction) -> &str {
    let category = tx.category.trim();
    if category.is_empty() {
        "Uncategorized"
    } else {
        category
    }
}

fn encode_filter_value(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in value.trim().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push('%');
                encoded.push(HEX[(byte >> 4) as usize] as char);
                encoded.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    encoded
}

fn decode_filter_value(value: &str) -> String {
    let mut decoded = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                if let (Some(high), Some(low)) =
                    (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
                {
                    decoded.push((high << 4) | low);
                    index += 3;
                } else {
                    decoded.push(bytes[index]);
                    index += 1;
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).unwrap_or_else(|_| value.replace('+', " "))
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_month_filter(value: &str) -> Option<MonthKey> {
    let (year, month) = value.split_once('-')?;
    Some(MonthKey::new(year.parse().ok()?, month.parse().ok()?))
}

pub(in crate::app) fn show_transactions_filter(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    filter: TransactionFilter,
) {
    let query = filter.query();
    ui.stack.set_visible_child_name("transactions");
    *ui.active_transaction_filter.borrow_mut() = Some(filter);
    *ui.search_query.borrow_mut() = query.clone();

    if query.is_empty() {
        ui.search_bar.set_search_mode(false);
    } else {
        ui.search_bar.set_search_mode(true);
    }

    if ui.search_entry.text().as_str() != query.as_str() {
        ui.search_entry.set_text(&query);
    }
    render_views(&state.borrow(), ui, state);
    if query.is_empty() {
        show_status(ui, "Filter cleared. All items are shown.");
    } else {
        show_status(
            ui,
            &trf(
                "Filter active: “{query}”. Clear the search text to show everything.",
                &[("query", query.clone())],
            ),
        );
    }
}

#[derive(Debug, Clone)]
pub(in crate::app) struct SearchFilter {
    pub(in crate::app) raw: String,
    needle: String,
    transaction_filter: Option<TransactionFilter>,
}

impl SearchFilter {
    pub(in crate::app) fn from_text(text: &str) -> Option<Self> {
        Self::from_text_with_transaction_filter(text, None)
    }

    fn from_text_with_transaction_filter(
        text: &str,
        transaction_filter: Option<TransactionFilter>,
    ) -> Option<Self> {
        let raw = text.trim().to_string();
        if raw.is_empty() {
            None
        } else {
            Some(Self {
                needle: raw.to_lowercase(),
                transaction_filter: transaction_filter
                    .or_else(|| TransactionFilter::from_query(&raw)),
                raw,
            })
        }
    }

    pub(in crate::app) fn matches(&self, text: &str) -> bool {
        text.to_lowercase().contains(&self.needle)
    }

    pub(in crate::app) fn matches_transaction(
        &self,
        tx: &Transaction,
        budgets: &[BudgetCode],
    ) -> bool {
        self.transaction_filter
            .as_ref()
            .map(|filter| filter.matches(tx, budgets))
            .unwrap_or_else(|| self.matches(&transaction_search_text(tx)))
    }
}

pub(in crate::app) fn active_search(ui: &UiHandles) -> Option<SearchFilter> {
    let query = ui.search_query.borrow().clone();
    let transaction_filter = ui.active_transaction_filter.borrow().clone();
    SearchFilter::from_text_with_transaction_filter(&query, transaction_filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_filter_with_year_preserves_month_and_details() {
        let filter = TransactionFilter::Scoped {
            budget_code: Some("FOOD".to_string()),
            year: None,
            month: Some(MonthKey::new(2025, 5)),
            amount: Some(TransactionAmountFilter::Expense),
        };

        let shifted = filter.with_year(2024).unwrap();

        assert!(matches!(
            shifted,
            TransactionFilter::Scoped {
                budget_code: Some(ref code),
                year: None,
                month: Some(MonthKey { year: 2024, month: 5 }),
                amount: Some(TransactionAmountFilter::Expense),
            } if code == "FOOD"
        ));
    }

    #[test]
    fn transaction_filter_with_year_preserves_category_filter() {
        let filter = TransactionFilter::category_for_year("Groceries", 2025);

        let shifted = filter.with_year(2024).unwrap();

        assert!(matches!(
            shifted,
            TransactionFilter::CategoryForYear { ref category, year: 2024 }
                if category == "Groceries"
        ));
    }

    #[test]
    fn diagnostic_transaction_filters_keep_hidden_rows_available() {
        assert!(TransactionFilter::UnconfiguredBudgets.includes_diagnostic_hidden_rows());
        assert!(TransactionFilter::OtherCategories.includes_diagnostic_hidden_rows());
        assert!(!TransactionFilter::All.includes_diagnostic_hidden_rows());
        assert!(!TransactionFilter::year(2025).includes_diagnostic_hidden_rows());
    }

    #[test]
    fn diagnostics_page_does_not_hide_canceled_transactions() {
        assert!(!page_hides_canceled_transactions(
            AppPage::Diagnostics,
            true,
            None,
        ));
    }

    #[test]
    fn diagnostic_transaction_filters_do_not_hide_canceled_transactions() {
        assert!(!page_hides_canceled_transactions(
            AppPage::Transactions,
            true,
            Some(&TransactionFilter::UnconfiguredBudgets),
        ));
        assert!(page_hides_canceled_transactions(
            AppPage::Transactions,
            true,
            Some(&TransactionFilter::year(2025)),
        ));
    }

    #[test]
    fn ordinary_pages_honor_hide_canceled_preference() {
        assert!(page_hides_canceled_transactions(
            AppPage::Overview,
            true,
            None,
        ));
        assert!(!page_hides_canceled_transactions(
            AppPage::Budget,
            false,
            None,
        ));
    }
}
