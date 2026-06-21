use super::super::{render_views, show_status, trf, AppData, UiHandles};
use super::TransactionFilter;
use crate::app::transactions::transaction_search_text;
use crate::model::{BudgetCode, Transaction};
use adw::gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

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
        show_search_status(&ui_for_search, &query);
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

pub(in crate::app) fn show_transactions_filter(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    filter: TransactionFilter,
) {
    let query = filter.query();
    show_transaction_search(state, ui, &query, Some(filter));
}

pub(in crate::app) fn show_transaction_search(
    state: &Rc<RefCell<AppData>>,
    ui: &Rc<UiHandles>,
    query: &str,
    transaction_filter: Option<TransactionFilter>,
) {
    ui.stack.set_visible_child_name("transactions");
    *ui.active_transaction_filter.borrow_mut() = transaction_filter;
    *ui.search_query.borrow_mut() = query.to_string();
    ui.search_bar.set_search_mode(!query.is_empty());

    if ui.search_entry.text().as_str() != query {
        ui.search_entry.set_text(query);
    }
    render_views(&state.borrow(), ui, state);
    show_search_status(ui, query);
}

pub(super) fn show_search_status(ui: &UiHandles, query: &str) {
    if query.is_empty() {
        show_status(ui, "Filter cleared. All items are shown.");
    } else {
        show_status(
            ui,
            &trf(
                "Filter active: “{query}”. Clear the search text to show everything.",
                &[("query", query.to_string())],
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

    pub(in crate::app) fn matches_summary(&self, text: &str) -> bool {
        self.transaction_filter.is_some() || self.matches(text)
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
