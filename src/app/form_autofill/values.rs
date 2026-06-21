use super::entries::budget_autofill_entries;
use super::*;

pub(in crate::app) fn app_category_values(data: &AppData) -> Vec<String> {
    unique_texts(
        data.budgets
            .iter()
            .map(|budget| budget.category.trim().to_string()),
    )
}

pub(in crate::app) fn app_budget_code_values(data: &AppData) -> Vec<String> {
    unique_texts(
        data.budgets
            .iter()
            .map(|budget| budget.code.trim().to_string()),
    )
}

pub(in crate::app) fn app_budget_autofill_entries(data: &AppData) -> Vec<BudgetAutofillEntry> {
    budget_autofill_entries(data.budgets.iter().map(|budget| {
        (
            budget.code.clone(),
            budget.category.clone(),
            budget.direction.as_str().to_string(),
        )
    }))
}

pub(in crate::app) fn transaction_rule_search_values(tx: &Transaction) -> Vec<String> {
    unique_texts([
        tx.counterparty.trim().to_string(),
        tx.tags.trim().to_string(),
        tx.description.trim().to_string(),
        tx.account.trim().to_string(),
        tx.transaction_id.trim().to_string(),
    ])
}

pub(in crate::app) fn editable_category_values() -> Vec<String> {
    unique_texts(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| budget.category),
    )
}

pub(in crate::app) fn editable_budget_code_values() -> Vec<String> {
    unique_texts(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| budget.code),
    )
}

pub(in crate::app) fn editable_budget_autofill_entries() -> Vec<BudgetAutofillEntry> {
    budget_autofill_entries(
        data::load_editable_budgets()
            .unwrap_or_default()
            .into_iter()
            .map(|budget| (budget.code, budget.category, budget.direction)),
    )
}

pub(in crate::app) fn editable_rule_search_values() -> Vec<String> {
    unique_texts(
        data::load_editable_rules()
            .unwrap_or_default()
            .into_iter()
            .map(|rule| rule.search),
    )
}

fn unique_texts(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort_by_key(|value| value.to_ascii_uppercase());
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
}
