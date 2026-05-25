use super::*;

pub(super) fn transaction_title(tx: &Transaction) -> String {
    let counterparty = tx.counterparty.trim();
    if counterparty.is_empty() {
        tx.description.trim().to_string()
    } else {
        counterparty.to_string()
    }
}

pub(super) fn transaction_subtitle(tx: &Transaction, advanced_features: bool) -> String {
    let title = transaction_title(tx);
    let mut parts = vec![tx.date.to_string()];
    push_subtitle_part(&mut parts, &tx.category);
    if advanced_features {
        push_subtitle_part(&mut parts, &tx.budget_code);
    }
    push_subtitle_part(&mut parts, &tx.tags);

    if !same_subtitle_value(&title, &tx.description) {
        push_subtitle_part(&mut parts, &tx.description);
    }
    parts.join(" · ")
}

fn push_subtitle_part(parts: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        parts.push(value.to_string());
    }
}

fn same_subtitle_value(left: &str, right: &str) -> bool {
    let left = crate::util::normalize_key(left);
    let right = crate::util::normalize_key(right);
    !left.is_empty() && left == right
}

pub(crate) fn transaction_search_text(tx: &Transaction) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
        tx.date,
        signed_money(tx.amount),
        tx.amount,
        tx.description,
        tx.counterparty,
        tx.tags,
        tx.account,
        tx.transaction_id,
        tx.currency,
        tx.source_file,
        tx.source_row,
        tx.category,
        tx.budget_code,
        tx.notes,
        tx.strict_key,
        tx.loose_key,
    )
}

pub(super) fn markup_escape(text: &str) -> String {
    adw::glib::markup_escape_text(text).to_string()
}
