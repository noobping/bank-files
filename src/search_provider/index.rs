use super::data::load_search_data;
use crate::app::transaction_search_text;
use crate::app_info::APP_ID;
use crate::model::{AppData, Transaction};
use crate::util::signed_money;

use adw::glib::variant::ToVariant;
use adw::glib::Variant;
use sha2::{Digest, Sha256};

use std::collections::HashMap;

const RESULT_ID_SEPARATOR: char = '\u{1f}';

pub(super) fn fallback_meta(identifier: &str) -> HashMap<String, Variant> {
    let mut meta = HashMap::new();
    meta.insert("id".to_string(), identifier.to_variant());
    meta.insert("name".to_string(), identifier.to_variant());
    meta.insert("gicon".to_string(), APP_ID.to_variant());
    meta
}

pub(super) fn encode_result_id(tx: &Transaction) -> String {
    let mut digest = Sha256::new();
    digest.update(tx.source_file.as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.source_row.to_string().as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.strict_key.as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.date.to_string().as_bytes());
    digest.update([RESULT_ID_SEPARATOR as u8]);
    digest.update(tx.amount.to_string().as_bytes());
    digest
        .finalize()
        .into_iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn result_id_is_valid(identifier: &str) -> bool {
    identifier.len() == 64 && identifier.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn transaction_title(tx: &Transaction) -> String {
    first_present([
        tx.counterparty.as_str(),
        tx.description.as_str(),
        tx.category.as_str(),
        tx.budget_code.as_str(),
    ])
    .unwrap_or("Transaction")
    .to_string()
}

fn transaction_description(tx: &Transaction) -> String {
    [
        tx.date.to_string(),
        signed_money(tx.amount),
        tx.category.trim().to_string(),
        tx.description.trim().to_string(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

fn first_present<'a>(values: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    values
        .into_iter()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn transaction_activation_query(tx: &Transaction) -> String {
    first_present([tx.transaction_id.as_str(), tx.strict_key.as_str()])
        .map(str::to_string)
        .unwrap_or_else(|| {
            [
                tx.source_file.trim().to_string(),
                tx.source_row.to_string(),
                tx.date.to_string(),
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
        })
}

#[derive(Default)]
pub(super) struct SearchProviderIndex {
    entries: Vec<SearchProviderEntry>,
}

impl SearchProviderIndex {
    pub(super) fn load() -> Self {
        Self::from_data(&load_search_data())
    }

    pub(super) fn from_data(data: &AppData) -> Self {
        Self {
            entries: data
                .transactions
                .iter()
                .map(SearchProviderEntry::from_transaction)
                .collect(),
        }
    }

    pub(super) fn search_result_ids(&self, terms: &[String], limit: usize) -> Vec<String> {
        let terms = normalized_search_terms(terms);
        if terms.is_empty() {
            return Vec::new();
        }

        self.entries
            .iter()
            .filter(|entry| entry.matches_terms(&terms))
            .take(limit)
            .map(|entry| entry.identifier.clone())
            .collect()
    }

    fn entry_for_identifier(&self, identifier: &str) -> Option<&SearchProviderEntry> {
        if !result_id_is_valid(identifier) {
            return None;
        }

        self.entries
            .iter()
            .find(|entry| entry.identifier == identifier)
    }

    pub(super) fn meta_for_identifier(&self, identifier: &str) -> Option<HashMap<String, Variant>> {
        let entry = self.entry_for_identifier(identifier)?;
        let mut meta = HashMap::new();
        meta.insert("id".to_string(), entry.identifier.to_variant());
        meta.insert("name".to_string(), entry.title.to_variant());
        meta.insert("description".to_string(), entry.description.to_variant());
        meta.insert("gicon".to_string(), APP_ID.to_variant());
        Some(meta)
    }

    pub(super) fn activation_query_for_identifier(&self, identifier: &str) -> Option<String> {
        self.entry_for_identifier(identifier)
            .map(|entry| entry.activation_query.clone())
    }
}

pub(super) struct SearchProviderEntry {
    identifier: String,
    search_text: String,
    title: String,
    description: String,
    activation_query: String,
}

impl SearchProviderEntry {
    pub(super) fn from_transaction(tx: &Transaction) -> Self {
        Self {
            identifier: encode_result_id(tx),
            search_text: transaction_search_text(tx).to_lowercase(),
            title: transaction_title(tx),
            description: transaction_description(tx),
            activation_query: transaction_activation_query(tx),
        }
    }

    pub(super) fn matches_terms(&self, terms: &[String]) -> bool {
        terms.iter().all(|term| self.search_text.contains(term))
    }
}

pub(super) fn normalized_search_terms(terms: &[String]) -> Vec<String> {
    terms
        .iter()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect()
}
