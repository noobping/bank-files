use super::super::*;
use super::model::FakeTransaction;

pub(super) const EMPTY_FAKE_TRANSACTIONS_SEARCH_TEXT: &str = "No matching fake transactions.";
pub(super) const EMPTY_FAKE_TRANSACTIONS_TEXT: &str = "No fake transactions.";
pub(super) const FAKE_TRANSACTIONS_TITLE: &str = "Fake Transactions";

pub(super) fn fake_transaction_search_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(str::trim)
        .filter(|term| !term.is_empty())
        .map(str::to_lowercase)
        .collect()
}

pub(super) fn fake_transaction_matches_search(fake: &FakeTransaction, terms: &[String]) -> bool {
    if terms.is_empty() {
        return true;
    }

    let transaction = &fake.transaction;
    let haystack = format!(
        "{} {} {} {} {} {}",
        fake_transaction_title(transaction),
        fake_transaction_subtitle(transaction),
        signed_money(transaction.amount),
        transaction.account,
        transaction.tags,
        transaction.notes
    )
    .to_lowercase();
    terms.iter().all(|term| haystack.contains(term))
}

pub(super) fn fake_transaction_title(transaction: &Transaction) -> String {
    let title = if transaction.counterparty.trim().is_empty() {
        transaction.description.trim()
    } else {
        transaction.counterparty.trim()
    };
    if title.is_empty() {
        tr("Fake transaction")
    } else {
        title.to_string()
    }
}

pub(super) fn fake_transaction_subtitle(transaction: &Transaction) -> String {
    format!(
        "{} · {} · {} · {}",
        transaction.date, transaction.category, transaction.budget_code, transaction.description
    )
}

pub(super) fn fake_transaction_summary(count: usize) -> String {
    trf(
        "{count} fake transaction(s) affect this session.",
        &[("count", count.to_string())],
    )
}
