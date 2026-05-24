use crate::model::Transaction;

use std::collections::HashSet;

pub(super) fn labels_related(left: &str, right: &str) -> bool {
    !left.is_empty()
        && !right.is_empty()
        && (left == right
            || (left.len() >= 4 && right.contains(left))
            || (right.len() >= 4 && left.contains(right)))
}

pub(super) fn looks_like_refund(transaction: &Transaction) -> bool {
    let text = format!(
        "{} {} {} {}",
        transaction.counterparty, transaction.description, transaction.tags, transaction.notes
    )
    .to_ascii_lowercase();
    [
        "refund",
        "reversal",
        "retour",
        "terug",
        "restitutie",
        "erstattung",
    ]
    .iter()
    .any(|word| text.contains(word))
}

pub(super) fn normalized_label(transaction: &Transaction) -> String {
    transaction_pattern_labels(transaction)
        .into_iter()
        .map(|label| normalized_text(&label))
        .find(|label| !label.is_empty())
        .unwrap_or_default()
}

pub(super) fn transaction_pattern_pair_match_labels(
    left: &Transaction,
    right: &Transaction,
) -> Vec<String> {
    transaction_pattern_match_labels(left)
        .into_iter()
        .chain(transaction_pattern_match_labels(right))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn transaction_pattern_match_labels(transaction: &Transaction) -> Vec<String> {
    transaction_pattern_labels(transaction)
        .into_iter()
        .map(|label| normalized_text(&label))
        .filter(|label| !label.is_empty())
        .collect()
}

pub(super) fn transaction_pattern_labels(transaction: &Transaction) -> Vec<String> {
    let mut labels = Vec::new();
    push_label(&mut labels, &transaction.tags);
    push_label(&mut labels, &transaction.counterparty);
    push_label(&mut labels, &description_tag_text(&transaction.description));
    push_label(&mut labels, &transaction.description);
    push_label(&mut labels, &transaction.budget_code);
    push_label(&mut labels, &transaction.category);
    labels
}

fn push_label(labels: &mut Vec<String>, label: &str) {
    let label = label.trim();
    if label.is_empty() {
        return;
    }
    let normalized = normalized_text(label);
    if normalized.is_empty()
        || labels
            .iter()
            .any(|existing| normalized_text(existing) == normalized)
    {
        return;
    }
    labels.push(label.to_string());
}

pub(super) fn normalized_text(raw: &str) -> String {
    raw.chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| meaningful_token(token))
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
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
    if digits == 0 {
        return true;
    }
    let len = token.chars().count();
    digits * 2 < len && len <= 18
}

pub(super) fn display_label(transaction: &Transaction) -> String {
    transaction_pattern_labels(transaction)
        .into_iter()
        .find(|label| !normalized_text(label).is_empty())
        .unwrap_or_else(|| "Transaction".to_string())
}

pub(super) fn description_tag_text(description: &str) -> String {
    normalized_text(description)
}

pub(super) fn transfer_pair_score(left: &Transaction, right: &Transaction) -> usize {
    let mut score = 0;
    let left_account = normalized_text(&left.account);
    let right_account = normalized_text(&right.account);
    if !left_account.is_empty() && !right_account.is_empty() && left_account != right_account {
        score += 2;
    }
    if looks_like_transfer(left) || looks_like_transfer(right) {
        score += 2;
    }
    if labels_related(&normalized_label(left), &normalized_label(right)) {
        score += 1;
    }
    score
}

fn looks_like_transfer(transaction: &Transaction) -> bool {
    let text = normalized_text(&format!(
        "{} {} {} {} {}",
        transaction.account,
        transaction.counterparty,
        transaction.description,
        transaction.tags,
        transaction.notes
    ));
    [
        "overboeking",
        "overboekingen",
        "transfer",
        "transfers",
        "sparen",
        "spaarrekening",
        "savings",
        "saving",
        "internal",
        "eigen rekening",
        "rekening",
        "wise",
        "revolut",
        "bunq",
        "paypal",
        "ueberweisung",
        "uberweisung",
        "umbuchung",
    ]
    .iter()
    .any(|word| text.contains(word))
}

pub(super) fn transfer_label(left: &Transaction, right: &Transaction) -> String {
    [
        left.counterparty.trim(),
        right.counterparty.trim(),
        description_tag_text(&left.description).trim(),
        description_tag_text(&right.description).trim(),
        left.account.trim(),
        right.account.trim(),
    ]
    .into_iter()
    .find(|label| !label.is_empty())
    .unwrap_or("Transfer")
    .to_string()
}
