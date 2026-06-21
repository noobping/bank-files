use super::*;

mod connect;
mod entries;
mod inference;
mod values;

pub(in crate::app) use connect::connect_budget_fields_autofill;
pub(in crate::app) use values::{
    app_budget_autofill_entries, app_budget_code_values, app_category_values,
    editable_budget_autofill_entries, editable_budget_code_values, editable_category_values,
    editable_rule_search_values, transaction_rule_search_values,
};

#[derive(Debug, Clone)]
pub(in crate::app) struct BudgetAutofillEntry {
    code: String,
    category: String,
    direction: String,
}

#[derive(Debug, Clone, Copy)]
enum BudgetAutofillSource {
    Initial,
    Category,
    Code,
    Direction,
}

#[derive(Debug, Default)]
struct BudgetAutofillState {
    applying: Cell<bool>,
}
