use super::*;

mod annual;
mod cash_flow;
mod totals;

pub use annual::{annual_budget_usage, category_totals_for_year_comparison};
pub use cash_flow::cash_flow_breakdown_for_year;
pub use totals::{category_totals_for_month, category_totals_for_year};

pub(super) fn category_label(tx: &Transaction) -> &str {
    let category = tx.category.trim();
    if category.is_empty() {
        "Uncategorized"
    } else {
        category
    }
}
