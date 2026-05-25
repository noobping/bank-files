use super::super::{filtered_transactions, AppData, UiHandles};
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
    page_filtered_app_data(data, search.as_ref(), transaction_filter.as_ref())
}

pub(in crate::app) fn page_data_for_render(
    data: &AppData,
    transaction_filter: Option<&TransactionFilter>,
) -> Option<AppData> {
    page_filtered_app_data(data, None, transaction_filter)
}

fn page_filtered_app_data(
    data: &AppData,
    search: Option<&SearchFilter>,
    transaction_filter: Option<&TransactionFilter>,
) -> Option<AppData> {
    let owned_filter;
    let filter = if let Some(search) = search {
        Some(search)
    } else if let Some(transaction_filter) = transaction_filter {
        owned_filter = SearchFilter::from_text(&transaction_filter.query());
        owned_filter.as_ref()
    } else {
        None
    }?;

    let transactions = filtered_transactions(&data.transactions, &data.budgets, Some(filter))
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    if transactions.len() == data.transactions.len() {
        return None;
    }

    let mut filtered = data.clone();
    filtered.transactions = transactions;
    Some(filtered)
}
