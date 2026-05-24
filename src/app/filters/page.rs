use super::super::{
    analytics, effective_hide_canceled_transactions, filtered_transactions, AppData, UiHandles,
};
use super::{active_search, SearchFilter, TransactionFilter};
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(in crate::app) enum AppPage {
    Overview,
    Budget,
    Transactions,
    Diagnostics,
}

impl AppPage {
    pub(super) fn from_stack_name(name: Option<&str>) -> Self {
        match name {
            Some("categories") => Self::Budget,
            Some("transactions") => Self::Transactions,
            Some("debug") => Self::Diagnostics,
            _ => Self::Overview,
        }
    }

    pub(super) fn stack_name(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Budget => "categories",
            Self::Transactions => "transactions",
            Self::Diagnostics => "debug",
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
            ui.advanced_features.get(),
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

pub(super) fn page_hides_canceled_transactions(
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
