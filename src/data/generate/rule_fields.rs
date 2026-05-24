use super::super::*;
use super::budget_code::clean_label;
use crate::analytics;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum RuleField {
    Counterparty,
    Tags,
    Description,
}

impl RuleField {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Counterparty => "counterparty",
            Self::Tags => "tags",
            Self::Description => "description",
        }
    }
}

pub(super) struct RuleCandidate {
    pub(super) field: RuleField,
    pub(super) label: String,
}

pub(super) fn rule_candidate(transaction: &Transaction) -> Option<RuleCandidate> {
    [
        (RuleField::Counterparty, transaction.counterparty.as_str()),
        (RuleField::Tags, transaction.tags.as_str()),
        (RuleField::Description, transaction.description.as_str()),
    ]
    .into_iter()
    .find_map(|(field, label)| {
        let label = clean_label(label);
        meaningful_label(&label).then_some(RuleCandidate { field, label })
    })
}

pub(super) fn best_pattern_labels(
    pattern: &analytics::TransactionPattern,
    transactions: &[Transaction],
) -> Vec<String> {
    let matched = transactions
        .iter()
        .filter(|transaction| analytics::transaction_matches_pattern(transaction, pattern))
        .collect::<Vec<_>>();
    let mut labels = matched
        .iter()
        .filter_map(|transaction| rule_candidate(transaction).map(|candidate| candidate.label))
        .collect::<Vec<_>>();
    if labels.is_empty() {
        labels.extend(pattern.match_labels.iter().cloned());
    }
    labels
}

pub(super) fn rule_search_from_labels(labels: &[String]) -> String {
    if labels.len() == 1 {
        return labels[0].trim().to_string();
    }
    let terms = labels
        .iter()
        .map(|label| regex_term(label))
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    format!("(?:{})", terms.join("|"))
}

fn regex_term(label: &str) -> String {
    normalize_key(label)
        .split_whitespace()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join(r"\W+")
}

fn meaningful_label(label: &str) -> bool {
    let normalized = normalize_key(label);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }
    let meaningful = tokens
        .iter()
        .filter(|token| meaningful_token(token))
        .count();
    meaningful > 0
}

fn meaningful_token(token: &str) -> bool {
    const NOISE: &[&str] = &[
        "afschrijving",
        "betaling",
        "beschrijving",
        "card",
        "description",
        "id",
        "ideal",
        "iban",
        "incasso",
        "kenmerk",
        "machtiging",
        "mandate",
        "message",
        "nummer",
        "omschrijving",
        "pas",
        "payment",
        "reference",
        "ref",
        "sepa",
        "transaction",
        "transactie",
    ];
    if token.len() <= 1 || NOISE.contains(&token) {
        return false;
    }
    let digits = token
        .chars()
        .filter(|character| character.is_ascii_digit())
        .count();
    digits == 0 || digits * 2 < token.chars().count()
}
