use super::{LocalAiInput, LocalAiRecord};
use crate::model::{AppData, Transaction};
use crate::util::normalize_key;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

impl LocalAiInput {
    pub fn from_app_data(data: &AppData, locale: impl Into<String>) -> Self {
        let mut groups = BTreeMap::<(String, String), LocalAiRecordBuilder>::new();
        for transaction in &data.transactions {
            let Some(label) = transaction_label(transaction) else {
                continue;
            };
            let direction = transaction_direction(transaction.amount).to_string();
            groups
                .entry((normalize_key(&label), direction.clone()))
                .or_insert_with(|| LocalAiRecordBuilder::new(label, direction))
                .push(transaction);
        }

        let records = groups
            .into_values()
            .map(LocalAiRecordBuilder::finish)
            .collect::<Vec<_>>();

        Self {
            locale: locale.into(),
            records,
        }
    }
}

#[derive(Debug, Clone)]
struct LocalAiRecordBuilder {
    label: String,
    descriptions: BTreeSet<String>,
    tags: BTreeSet<String>,
    direction: String,
    existing_category: String,
    existing_budget_code: String,
    count: usize,
}

impl LocalAiRecordBuilder {
    fn new(label: String, direction: String) -> Self {
        Self {
            label,
            descriptions: BTreeSet::new(),
            tags: BTreeSet::new(),
            direction,
            existing_category: String::new(),
            existing_budget_code: String::new(),
            count: 0,
        }
    }

    fn push(&mut self, transaction: &Transaction) {
        self.count += 1;
        push_sanitized(&mut self.descriptions, &transaction.description, 4);
        for tag in transaction.tags.split([',', ';', '|']) {
            push_sanitized(&mut self.tags, tag, 6);
        }
        if self.existing_category.is_empty() && !transaction.category.trim().is_empty() {
            self.existing_category = transaction.category.trim().to_string();
        }
        if self.existing_budget_code.is_empty() && !transaction.budget_code.trim().is_empty() {
            self.existing_budget_code = transaction.budget_code.trim().to_string();
        }
    }

    fn finish(self) -> LocalAiRecord {
        LocalAiRecord {
            label: self.label,
            descriptions: self.descriptions.into_iter().collect(),
            tags: self.tags.into_iter().collect(),
            direction: self.direction,
            existing_category: self.existing_category,
            existing_budget_code: self.existing_budget_code,
            count: self.count,
        }
    }
}

fn push_sanitized(target: &mut BTreeSet<String>, input: &str, limit: usize) {
    if target.len() >= limit {
        return;
    }
    let value = sanitize_text(input);
    if !value.is_empty() {
        target.insert(value);
    }
}

fn sanitize_text(input: &str) -> String {
    input
        .split_whitespace()
        .filter(|token| !sensitive_token(token))
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect::<String>()
        .trim()
        .to_string()
}

fn sensitive_token(token: &str) -> bool {
    let digits = token.chars().filter(|ch| ch.is_ascii_digit()).count();
    digits >= 6 || token.contains('@') || token.starts_with("IBAN") || token.starts_with("iban")
}

fn transaction_label(transaction: &Transaction) -> Option<String> {
    [
        transaction.counterparty.trim(),
        transaction.description.trim(),
        transaction.tags.trim(),
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .map(sanitize_text)
    .filter(|value| !value.is_empty())
}

fn transaction_direction(amount: Decimal) -> &'static str {
    if amount > Decimal::ZERO {
        "income"
    } else if amount < Decimal::ZERO {
        "expense"
    } else {
        "any"
    }
}
