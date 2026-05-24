use super::*;

pub(super) fn transaction_title(tx: &Transaction) -> String {
    if tx.counterparty.trim().is_empty() {
        tx.description.clone()
    } else {
        tx.counterparty.clone()
    }
}

pub(super) fn transaction_subtitle(tx: &Transaction) -> String {
    if tx.tags.trim().is_empty() {
        format!(
            "{} · {} · {} · {}",
            tx.date, tx.category, tx.budget_code, tx.description
        )
    } else {
        format!(
            "{} · {} · {} · {} · {}",
            tx.date, tx.category, tx.budget_code, tx.tags, tx.description
        )
    }
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
