mod apply;
mod defaults;
mod fallback;
mod load;
mod model;
mod text;

pub use apply::{apply_rules, transaction_classification_is_auto_detected};
pub use load::{load_budget_codes, load_rules};
pub use model::Rule;

#[cfg(test)]
mod tests;
