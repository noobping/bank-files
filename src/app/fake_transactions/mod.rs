mod controller;
mod form;
mod list;
mod model;
mod presentation;
mod transaction_builder;
mod widgets;

const FAKE_TRANSACTION_SOURCE: &str = "Runtime fake transaction";
const DEFAULT_FAKE_ACCOUNT: &str = "Fake";
const DEFAULT_FAKE_CURRENCY: &str = "EUR";
const FAKE_TRANSACTIONS_LIST_PAGE: &str = "list";
const FAKE_TRANSACTIONS_FORM_PAGE: &str = "form";

pub(in crate::app) use controller::{connect_fake_transactions, duplicate_transaction_as_fake};
pub(in crate::app) use model::{
    data_with_fake_transactions, real_transactions, transaction_is_fake, FakeTransactionStore,
};
pub(in crate::app) use widgets::{
    build_fake_transaction_widgets, focus_fake_transaction_search, FakeTransactionWidgets,
};

#[cfg(test)]
mod tests;
