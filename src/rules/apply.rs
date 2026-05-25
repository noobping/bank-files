use super::defaults::{canonical_direction, canonical_rule_field, AUTO_DETECTED_CATEGORY_NOTE};
use super::fallback::fallback_category;
use super::transaction_tag_text;
use super::Rule;
use crate::model::Transaction;
use crate::util::normalize_key;

use regex::RegexBuilder;
use rust_decimal::Decimal;

pub fn apply_rules(transactions: &mut [Transaction], rules: &[Rule]) {
    for tx in transactions {
        if apply_matching_rule(tx, rules) {
            continue;
        }

        let assignment = fallback_category(tx);
        tx.category = assignment.category;
        tx.budget_code = assignment.budget_code;
        if let Some(notes) = assignment.notes {
            tx.notes = notes;
        }
    }
}

pub fn transaction_classification_is_auto_detected(tx: &Transaction) -> bool {
    note_is_auto_detection(&tx.notes)
}

fn apply_matching_rule(tx: &mut Transaction, rules: &[Rule]) -> bool {
    for rule in rules.iter().filter(|rule| rule.active) {
        if rule_matches(rule, tx) {
            tx.category = rule.category.clone();
            tx.budget_code = rule.budget_code.clone();
            tx.notes = rule.notes.clone();
            return true;
        }
    }
    false
}

fn note_is_auto_detection(note: &str) -> bool {
    note_matches(note, AUTO_DETECTED_CATEGORY_NOTE)
}

fn note_matches(note: &str, expected: &str) -> bool {
    let note = normalize_key(note);
    let expected_key = normalize_key(expected);
    let localized_key = normalize_key(&crate::i18n::gettext(expected));
    !note.is_empty()
        && (note == expected_key
            || note.starts_with(&format!("{expected_key} "))
            || note == localized_key
            || note.starts_with(&format!("{localized_key} ")))
}

fn rule_matches(rule: &Rule, tx: &Transaction) -> bool {
    if !direction_matches(&rule.direction, tx.amount) {
        return false;
    }
    let abs = tx.amount.abs();
    if let Some(min) = rule.amount_min {
        if abs < min.abs() {
            return false;
        }
    }
    if let Some(max) = rule.amount_max {
        if abs > max.abs() {
            return false;
        }
    }

    let text = match canonical_rule_field(&rule.field) {
        Some("description") => tx.description.clone(),
        Some("counterparty") => tx.counterparty.clone(),
        Some("tags") => transaction_tag_text(tx),
        Some("account") => tx.account.clone(),
        Some("transaction id") => tx.transaction_id.clone(),
        _ => format!(
            "{} {} {} {} {}",
            tx.description,
            tx.counterparty,
            transaction_tag_text(tx),
            tx.account,
            tx.transaction_id
        ),
    };

    let Ok(re) = RegexBuilder::new(&rule.pattern)
        .case_insensitive(true)
        .build()
    else {
        return normalize_key(&text).contains(&normalize_key(&rule.pattern));
    };
    re.is_match(&text)
}

fn direction_matches(direction: &str, amount: Decimal) -> bool {
    let Some(direction) = canonical_direction(direction) else {
        return true;
    };
    match direction {
        "expense" => amount < Decimal::ZERO,
        "income" => amount > Decimal::ZERO,
        "transfer" => true,
        _ => true,
    }
}
