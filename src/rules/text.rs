use crate::model::Transaction;

pub(super) fn transaction_tag_text(tx: &Transaction) -> String {
    let tags = tx.tags.trim();
    let description = tx.description.trim();
    match (tags.is_empty(), description.is_empty()) {
        (true, true) => String::new(),
        (true, false) => description.to_string(),
        (false, true) => tags.to_string(),
        (false, false) => format!("{tags} {description}"),
    }
}
