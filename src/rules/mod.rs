mod apply;
mod defaults;
mod fallback;
mod load;

use crate::model::Transaction;
use rust_decimal::Decimal;

pub use apply::{apply_rules, transaction_classification_is_auto_detected};
pub use load::{load_budget_codes, load_rules};

#[derive(Debug, Clone)]
pub struct Rule {
    pub priority: i32,
    pub active: bool,
    pub field: String,
    pub pattern: String,
    pub category: String,
    pub budget_code: String,
    pub direction: String,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub notes: String,
}

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

#[cfg(test)]
mod tests;
